#![cfg(target_arch = "wasm32")]

use std::{
    collections::VecDeque,
    str::{self, FromStr},
};

use either::Either;

use futures_util::{future::AbortHandle, stream::Abortable, StreamExt};

use gloo_console::{error, info, warn};

use ipfs_api::IpfsService;

use linked_data::{
    live::LiveSettings,
    media::video::{Setup, Track, Video},
    types::PeerId,
};

use serde::{Deserialize, Serialize};

use utils::{ipfs::IPFSContext, seconds_to_timecode};

use wasm_bindgen::{closure::Closure, JsCast};

use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url};

use yew::{
    classes,
    platform::spawn_local,
    prelude::{html, Component, Html, Properties},
    Callback, Context,
};

use cid::Cid;

use crate::ema::ExponentialMovingAverage;

const FORWARD_BUFFER_LENGTH: f64 = 16.0;
const BACK_BUFFER_LENGTH: f64 = 8.0;

const SETUP_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup";

//Could build a state machine implicit in the type system instead of using callbacks

enum MachineState {
    Load,
    Switch,
    Flush,
    Timeout,
    AdaptativeBitrate,
    Status,
}

/// Deserialize either live settings or video metadata
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct LiveOrVideo {
    #[serde(with = "either::serde_untagged")]
    inner: Either<LiveSettings, Video>,
}

struct MediaBuffers {
    audio: SourceBuffer,
    video: SourceBuffer,

    tracks: Vec<Track>,
}

struct LiveStream {
    settings: LiveSettings,

    buffer: VecDeque<Cid>,

    handle: AbortHandle,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed Link to Media Cid
    pub cid: Cid,
}

/// Video player for live streams and on demand.
pub struct VideoPlayer {
    ipfs: IpfsService,

    player_type: Option<Either<LiveStream, Video>>,

    object_url: String,
    media_element: Option<HtmlMediaElement>,
    media_source: MediaSource,
    media_buffers: Option<MediaBuffers>,

    update_end_cb: Callback<()>,
    timeout_cb: Callback<()>,
    append_cb: Callback<(Vec<u8>, Vec<u8>)>,
    append_video_cb: Callback<Vec<u8>>,

    /// Level >= 1 since 0 is audio
    level: usize,
    state: MachineState,
    ema: ExponentialMovingAverage,
    source_open_closure: Option<Closure<dyn Fn()>>,
    seeking_closure: Option<Closure<dyn Fn()>>,
    update_end_closure: Option<Closure<dyn Fn()>>,
    timeout_closure: Option<Closure<dyn Fn()>>,
    handle: i32,
}

pub enum Msg {
    Settings(Either<LiveSettings, Video>),
    SourceOpen,
    Seeking,
    UpdateEnd,
    Timeout,
    SetupNode(Setup),
    Append((Vec<u8>, Vec<u8>)),
    AppendVideo(Vec<u8>),
    PubSub((PeerId, Vec<u8>)),
}

impl Component for VideoPlayer {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _handle) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let ipfs = context.client;

        /* spawn_local({
            let ipfs = ipfs.clone();
            let cb = ctx.link().callback(Msg::Settings);
            let cid = ctx.props().cid;

            async move {
                match ipfs.dag_get::<&str, LiveOrVideo>(cid, Some("/link")).await {
                    Ok(either) => cb.emit(either.inner),
                    Err(e) => {
                        error!(&format!("{:#?}", e));
                        return;
                    }
                }
            }
        }); */

        let ema = ExponentialMovingAverage::new();

        let media_source = match MediaSource::new() {
            Ok(media_source) => media_source,
            Err(e) => {
                error!(&format!("{:#?}", e));
                std::process::abort();
            }
        };

        let object_url = match Url::create_object_url_with_source(&media_source) {
            Ok(object_url) => object_url,
            Err(e) => {
                error!(&format!("{:#?}", e));
                std::process::abort();
            }
        };

        let cb = ctx.link().callback(|_| Msg::SourceOpen);
        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
        media_source.set_onsourceopen(Some(closure.as_ref().unchecked_ref()));
        let source_open_closure = Some(closure);

        Self {
            ipfs,

            player_type: None,

            media_element: None,
            media_source,
            media_buffers: None,
            object_url,

            update_end_cb: ctx.link().callback(|()| Msg::UpdateEnd),
            timeout_cb: ctx.link().callback(|()| Msg::Timeout),
            append_cb: ctx.link().callback(Msg::Append),
            append_video_cb: ctx.link().callback(Msg::AppendVideo),

            level: 1, // start at 1 since 0 is audio
            state: MachineState::Timeout,
            ema,
            source_open_closure,
            seeking_closure: None,
            update_end_closure: None,
            timeout_closure: None,
            handle: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Settings(set) => return self.on_settigns(ctx, set),
            Msg::SourceOpen => self.on_source_open(ctx),
            Msg::Seeking => self.on_seeking(),
            Msg::UpdateEnd => self.on_update_end(),
            Msg::Timeout => self.on_timeout(),
            Msg::SetupNode(setup) => self.add_source_buffer(setup),
            Msg::Append((aud, vid)) => self.append_buffers(aud, vid),
            Msg::AppendVideo(result) => self.append_video_buffer(result),
            Msg::PubSub((peer, data)) => self.on_pubsub_update(ctx, peer, data),
        }

        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ybc::Image size={ybc::ImageSize::Is16by9} >
                <video class={classes!("has-ratio")} src={self.object_url.clone()} width=640 height=360 id="video_player" autoplay=true controls=true />
            </ybc::Image>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let window = match web_sys::window() {
                Some(window) => window,
                None => {
                    #[cfg(debug_assertions)]
                    error!("No Window Object");
                    return;
                }
            };

            let document = match window.document() {
                Some(document) => document,
                None => {
                    #[cfg(debug_assertions)]
                    error!("No Document Object");
                    return;
                }
            };

            let element = match document.get_element_by_id("video_player") {
                Some(document) => document,
                None => {
                    #[cfg(debug_assertions)]
                    error!("No Element by Id");
                    return;
                }
            };

            let media_element: HtmlMediaElement = match element.dyn_into() {
                Ok(document) => document,
                Err(e) => {
                    error!(&format!("{:#?}", e));
                    return;
                }
            };

            self.seeking_closure = if let Some(Either::Right(_)) = self.player_type {
                let cb = ctx.link().callback(|()| Msg::Seeking);
                let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
                media_element.set_onseeking(Some(closure.as_ref().unchecked_ref()));

                Some(closure)
            } else {
                None
            };

            self.media_element = Some(media_element);
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(Either::Left(live)) = &mut self.player_type {
            live.handle.abort();
        }

        if let Some(window) = web_sys::window() {
            if self.handle != 0 {
                window.clear_timeout_with_handle(self.handle);
            }
        }
    }
}

impl VideoPlayer {
    fn on_settigns(&mut self, ctx: &Context<Self>, either: Either<LiveSettings, Video>) -> bool {
        #[cfg(debug_assertions)]
        info!("On Settings");

        let this = match either {
            Either::Left(settings) => {
                let (handle, regis) = AbortHandle::new_pair();

                let pubsub_cb = ctx.link().callback(Msg::PubSub);

                spawn_local({
                    let ipfs = self.ipfs.clone();
                    let cb = pubsub_cb.clone();
                    let topic = settings.video_topic.clone();

                    async move {
                        let stream = ipfs.pubsub_sub(topic.into_bytes());

                        let mut stream = Abortable::new(stream, regis).boxed_local();

                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(msg) => cb.emit((msg.from.into(), msg.data)),
                                Err(e) => error!(&format!("{:#?}", e)),
                            }
                        }
                    }
                });

                let live = LiveStream {
                    settings,
                    buffer: VecDeque::with_capacity(5),
                    handle,
                };

                Either::Left(live)
            }
            Either::Right(metadata) => {
                self.media_source.set_duration(metadata.duration);

                spawn_local({
                    let ipfs = self.ipfs.clone();
                    let cb = ctx.link().callback(Msg::SetupNode);
                    let cid = metadata.video.link;

                    async move {
                        match ipfs.dag_get::<&str, Setup>(cid, Some(SETUP_PATH)).await {
                            Ok(setup) => cb.emit(setup),
                            Err(e) => error!(&format!("{:#?}", e)),
                        }
                    }
                });

                Either::Right(metadata)
            }
        };

        self.player_type = Some(this);

        false
    }

    /// Callback when MediaSource is linked to video element.
    fn on_source_open(&mut self, ctx: &Context<Self>) {
        #[cfg(debug_assertions)]
        info!("On Source Open");

        self.media_source.set_onsourceopen(None);
        self.source_open_closure = None;

        spawn_local({
            let ipfs = self.ipfs.clone();
            let cb = ctx.link().callback(Msg::Settings);
            let cid = ctx.props().cid;

            async move {
                match ipfs.dag_get::<&str, LiveOrVideo>(cid, Some("/link")).await {
                    Ok(either) => cb.emit(either.inner),
                    Err(e) => {
                        error!(&format!("{:#?}", e));
                        return;
                    }
                }
            }
        });
    }

    /// Callback when GossipSub receive an update.
    fn on_pubsub_update(&mut self, ctx: &Context<Self>, from: PeerId, data: Vec<u8>) {
        #[cfg(debug_assertions)]
        info!("PubSub Message Received");

        let live = if let Some(Either::Left(live)) = &mut self.player_type {
            live
        } else {
            #[cfg(debug_assertions)]
            error!("No Live Stream");
            return;
        };

        #[cfg(debug_assertions)]
        info!(&format!("Sender => {}", from));

        if from != live.settings.peer_id {
            #[cfg(debug_assertions)]
            warn!("Unauthorized Sender");
            return;
        }

        let data = match str::from_utf8(&data) {
            Ok(data) => data,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return;
            }
        };

        #[cfg(debug_assertions)]
        info!(&format!("Message => {}", data));

        let cid = match Cid::from_str(data) {
            Ok(cid) => cid,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return;
            }
        };

        live.buffer.push_back(cid);

        if self.media_buffers.is_none() {
            spawn_local({
                let ipfs = self.ipfs.clone();
                let cb = ctx.link().callback(Msg::SetupNode);

                async move {
                    match ipfs.dag_get::<&str, Setup>(cid, Some("/setup")).await {
                        Ok(setup) => cb.emit(setup),
                        Err(e) => error!(&format!("{:#?}", e)),
                    }
                }
            })
        }
    }

    /// Callback when source buffer is done updating.
    fn on_update_end(&mut self) {
        #[cfg(debug_assertions)]
        info!("On Update End");

        self.tick()
    }

    /// Callback when video element has seeked.
    fn on_seeking(&mut self) {
        #[cfg(debug_assertions)]
        info!("On Seeking");

        self.state = MachineState::Flush;
    }

    /// Callback when 1 second has passed.
    fn on_timeout(&mut self) {
        #[cfg(debug_assertions)]
        info!("On Timeout");

        self.timeout_closure = None;
        self.handle = 0;

        self.tick()
    }

    /// Update state machine.
    fn tick(&mut self) {
        match self.state {
            MachineState::Load => self.load_segment(),
            MachineState::Switch => self.switch_quality(),
            MachineState::Flush => self.flush_buffer(),
            MachineState::Timeout => self.set_timeout(),
            MachineState::Status => self.check_status(),
            MachineState::AdaptativeBitrate => self.check_abr(),
        }
    }

    /// Set 1 second timeout.
    fn set_timeout(&mut self) {
        if self.timeout_closure.is_some() {
            return;
        }

        let cb = self.timeout_cb.clone();

        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);

        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                #[cfg(debug_assertions)]
                error!("No Window Object");
                return;
            }
        };

        match window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            1000,
        ) {
            Ok(handle) => self.handle = handle,
            Err(e) => error!(&format!("{:#?}", e)),
        }

        self.timeout_closure = Some(closure);
    }

    /// Create source buffer then load initialization segment.
    fn add_source_buffer(&mut self, setup_node: Setup) {
        #[cfg(debug_assertions)]
        info!("Adding Source Buffer");

        if self.media_source.ready_state() != MediaSourceReadyState::Open {
            #[cfg(debug_assertions)]
            info!("Media Source Not Open");
            return;
        }

        #[cfg(debug_assertions)]
        info!(&format!("Setup Node \n {:#?}", setup_node));

        #[cfg(debug_assertions)]
        info!("Listing Tracks");

        let mut audio_buffer = None;
        let mut video_buffer = None;

        for track in setup_node.tracks.iter() {
            if !MediaSource::is_type_supported(&track.codec) {
                error!(&format!("MIME Type {:?} unsupported", &track.codec));
                continue;
            }

            #[cfg(debug_assertions)]
            info!(&format!(
                "Name {} Codec {} Bandwidth {}",
                track.name, track.codec, track.bandwidth
            ));

            if video_buffer.is_some() {
                continue;
            }

            if track.name == "audio" && audio_buffer.is_some() {
                continue;
            }

            let source_buffer = match self.media_source.add_source_buffer(&track.codec) {
                Ok(sb) => sb,
                Err(e) => {
                    error!(&format!("{:#?}", e));
                    return;
                }
            };

            if track.name == "audio" {
                audio_buffer = Some(source_buffer);
            } else {
                video_buffer = Some(source_buffer);
            }
        }

        let audio = match audio_buffer {
            Some(audio) => audio,
            None => {
                #[cfg(debug_assertions)]
                error!("No Audio Buffer");
                return;
            }
        };

        let video = match video_buffer {
            Some(video) => video,
            None => {
                #[cfg(debug_assertions)]
                error!("No Video Buffer");
                return;
            }
        };

        let media_buffer = MediaBuffers {
            audio,
            video,
            tracks: setup_node.tracks,
        };

        let cb = self.update_end_cb.clone();
        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
        media_buffer
            .video
            .set_onupdateend(Some(closure.as_ref().unchecked_ref()));

        self.update_end_closure = Some(closure);

        let audio_cid = match media_buffer.tracks.get(0) {
            Some(track) => track.initialization_segment.link,
            None => {
                #[cfg(debug_assertions)]
                error!("No Track Index 0");
                return;
            }
        };

        let video_cid = match media_buffer.tracks.get(1) {
            Some(track) => track.initialization_segment.link,
            None => {
                #[cfg(debug_assertions)]
                error!("No Track Index 1");
                return;
            }
        };

        self.media_buffers = Some(media_buffer);
        self.state = MachineState::Load;

        spawn_local({
            let cb = self.append_cb.clone();
            let ipfs = self.ipfs.clone();

            async move {
                let aud_fut = ipfs.cat(audio_cid, Option::<&str>::None);
                let vid_fut = ipfs.cat(video_cid, Option::<&str>::None);

                let (aud_result, vid_result) = futures_util::join!(aud_fut, vid_fut);

                match (aud_result, vid_result) {
                    (Ok(aud), Ok(vid)) => cb.emit((aud.to_vec(), vid.to_vec())),
                    (Ok(_), Err(e)) => error!(&format!("{:#?}", e)),
                    (Err(e), Ok(_)) => error!(&format!("{:#?}", e)),
                    (Err(e), Err(r)) => {
                        error!(&format!("{:#?}", e));
                        error!(&format!("{:#?}", r));
                    }
                }
            }
        });
    }

    /// Load either live or VOD segment.
    fn load_segment(&mut self) {
        if let Some(Either::Right(_)) = self.player_type {
            return self.load_vod_segment();
        }

        if let Some(Either::Left(_)) = self.player_type {
            return self.load_live_segment();
        }
    }

    /// Try get cid from live buffer then fetch video data from ipfs.
    fn load_live_segment(&mut self) {
        let live = if let Some(Either::Left(live)) = &mut self.player_type {
            live
        } else {
            #[cfg(debug_assertions)]
            error!("No Live Stream");
            return;
        };

        let cid = match live.buffer.pop_front() {
            Some(cid) => cid,
            None => return self.set_timeout(),
        };

        #[cfg(debug_assertions)]
        info!("Loading Live Media Segments");

        let track_name = match self.media_buffers.as_ref() {
            Some(buf) => match buf.tracks.get(self.level) {
                Some(track) => &track.name,
                None => {
                    #[cfg(debug_assertions)]
                    error!("No Track");
                    return;
                }
            },
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        let audio_path = "/track/audio";
        let video_path = format!("/track/{}", track_name);

        self.state = MachineState::AdaptativeBitrate;
        self.ema.start_timer();

        spawn_local({
            let cb = self.append_cb.clone();
            let ipfs = self.ipfs.clone();

            async move {
                let aud_fut = ipfs.cat(cid, Some(audio_path));
                let vid_fut = ipfs.cat(cid, Some(video_path));

                let (aud_result, vid_result) = futures_util::join!(aud_fut, vid_fut);

                match (aud_result, vid_result) {
                    (Ok(aud), Ok(vid)) => cb.emit((aud.to_vec(), vid.to_vec())),
                    (Ok(_), Err(e)) => error!(&format!("{:#?}", e)),
                    (Err(e), Ok(_)) => error!(&format!("{:#?}", e)),
                    (Err(e), Err(r)) => {
                        error!(&format!("{:#?}", e));
                        error!(&format!("{:#?}", r));
                    }
                }
            }
        });
    }

    /// Get CID from timecode then fetch video data from ipfs.
    fn load_vod_segment(&mut self) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        let time_ranges = match buffers.video.buffered() {
            Ok(tm) => tm,
            Err(_) => {
                #[cfg(debug_assertions)]
                info!("Buffer empty");
                return;
            }
        };

        let mut buff_end = 0.0;

        let count = time_ranges.length();

        if count > 0 {
            if let Ok(end) = time_ranges.end(count - 1) {
                buff_end = end;
            }
        }

        //if buffer is empty load at current time
        if buff_end <= 0.0 {
            let current_time = match self.media_element.as_ref() {
                Some(media_element) => media_element.current_time(),
                None => {
                    #[cfg(debug_assertions)]
                    info!("No Media Element");
                    return;
                }
            };

            if current_time > 1.0 {
                buff_end = current_time - 1.0;
            }
        }

        let (hours, minutes, seconds) = seconds_to_timecode(buff_end);

        #[cfg(debug_assertions)]
        info!(&format!(
            "Loading Media Segments at timecode {}:{}:{}",
            hours, minutes, seconds
        ));

        let cid = if let Some(Either::Right(metadata)) = &self.player_type {
            metadata.video.link
        } else {
            #[cfg(debug_assertions)]
            error!("No Metadata");
            return;
        };

        let audio_path = format!(
            "/time/hour/{}/minute/{}/second/{}/video/track/audio",
            hours, minutes, seconds,
        );

        let track_name = match buffers.tracks.get(self.level) {
            Some(track) => &track.name,
            None => {
                #[cfg(debug_assertions)]
                error!("No Track");
                return;
            }
        };

        let video_path = format!(
            "/time/hour/{}/minute/{}/second/{}/video/track/{}",
            hours, minutes, seconds, track_name,
        );

        self.state = MachineState::AdaptativeBitrate;
        self.ema.start_timer();

        spawn_local({
            let cb = self.append_cb.clone();
            let ipfs = self.ipfs.clone();

            async move {
                let aud_fut = ipfs.cat(cid, Some(audio_path));
                let vid_fut = ipfs.cat(cid, Some(video_path));

                let (aud_result, vid_result) = futures_util::join!(aud_fut, vid_fut);

                match (aud_result, vid_result) {
                    (Ok(aud), Ok(vid)) => cb.emit((aud.to_vec(), vid.to_vec())),
                    (Ok(_), Err(e)) => error!(&format!("{:#?}", e)),
                    (Err(e), Ok(_)) => error!(&format!("{:#?}", e)),
                    (Err(e), Err(r)) => {
                        error!(&format!("{:#?}", e));
                        error!(&format!("{:#?}", r));
                    }
                }
            }
        });
    }

    /// Recalculate download speed then set quality level.
    fn check_abr(&mut self) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        let bandwidth = match buffers.tracks.get(self.level) {
            Some(track) => track.bandwidth as f64,
            None => {
                #[cfg(debug_assertions)]
                error!("No Track");
                return;
            }
        };

        let avg_bitrate = match self.ema.recalculate_average_speed(bandwidth) {
            Some(at) => at,
            None => {
                self.state = MachineState::Status;
                return self.tick();
            }
        };

        let mut next_level = 1; // start at 1 since 0 is audio
        while let Some(next_bitrate) = buffers.tracks.get(next_level + 1).map(|t| t.bandwidth) {
            if avg_bitrate <= next_bitrate as f64 {
                break;
            }

            next_level += 1;
        }

        if next_level == self.level {
            self.state = MachineState::Status;
            return self.tick();
        }

        self.level = next_level;
        self.state = MachineState::Switch;
        self.tick()
    }

    /// Check buffers and current time then trigger new action.
    fn check_status(&mut self) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        let time_ranges = match buffers.video.buffered() {
            Ok(tm) => tm,
            Err(_) => {
                #[cfg(debug_assertions)]
                info!("Buffer empty");
                return self.set_timeout();
            }
        };

        let count = time_ranges.length();

        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

        for i in 0..count {
            if let Ok(start) = time_ranges.start(i) {
                buff_start = start;
            }

            if let Ok(end) = time_ranges.end(i) {
                buff_end = end;
            }

            #[cfg(debug_assertions)]
            info!(&format!(
                "Time Range {} buffers {}s to {}s",
                i, buff_start, buff_end
            ));
        }

        let current_time = match self.media_element.as_ref() {
            Some(media_element) => media_element.current_time(),
            None => {
                #[cfg(debug_assertions)]
                info!("No Media Element");
                return self.set_timeout();
            }
        };

        if current_time < buff_start {
            let new_time = buff_start + ((buff_end - buff_start) / 2.0);

            #[cfg(debug_assertions)]
            info!(&format!("Forward To {}s", new_time));

            match self.media_element.as_ref() {
                Some(media_element) => media_element.set_current_time(new_time),
                None => {
                    #[cfg(debug_assertions)]
                    error!("No Media Element");
                    return;
                }
            }
        }

        if current_time > buff_start + BACK_BUFFER_LENGTH {
            #[cfg(debug_assertions)]
            info!("Back Buffer Full");
            return self.flush_buffer();
        }

        if let Some(Either::Right(metadata)) = &self.player_type {
            if buff_end >= metadata.duration {
                #[cfg(debug_assertions)]
                info!("End Of Video");
                return;
            }

            if current_time + FORWARD_BUFFER_LENGTH < buff_end {
                #[cfg(debug_assertions)]
                info!("Forward Buffer Full");
                return self.set_timeout();
            }
        }

        self.load_segment()
    }

    /// Flush everything or just back buffer.
    fn flush_buffer(&mut self) {
        #[cfg(debug_assertions)]
        info!("Flushing Buffer");

        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        let time_ranges = match buffers.video.buffered() {
            Ok(tm) => tm,
            Err(_) => {
                #[cfg(debug_assertions)]
                info!("Buffer empty");
                return;
            }
        };

        let count = time_ranges.length();

        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

        if let Ok(start) = time_ranges.start(0) {
            buff_start = start;
        }

        if let Ok(end) = time_ranges.end(count - 1) {
            buff_end = end;
        }

        let current_time = match self.media_element.as_ref() {
            Some(media_element) => media_element.current_time(),
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Element");
                return;
            }
        };

        let back_buffer_start = current_time - BACK_BUFFER_LENGTH;

        //full flush except if back buffer flush is possible
        if buff_start < back_buffer_start {
            buff_end = back_buffer_start
        }

        if let Err(e) = buffers.audio.remove(buff_start, buff_end) {
            error!(&format!("{:#?}", e));
            return;
        }

        if let Err(e) = buffers.video.remove(buff_start, buff_end) {
            error!(&format!("{:#?}", e));
            return;
        }

        self.state = MachineState::Load;
    }

    /// Switch source buffer codec then load initialization segment.
    fn switch_quality(&mut self) {
        #[cfg(debug_assertions)]
        info!("Switching Quality");

        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        let track = match buffers.tracks.get(self.level) {
            Some(track) => track,
            None => {
                #[cfg(debug_assertions)]
                error!("No Track");
                return;
            }
        };

        if let Err(e) = buffers.video.change_type(&track.codec) {
            error!(&format!("{:#?}", e));
            return;
        }

        #[cfg(debug_assertions)]
        info!(&format!(
            "Level {} Name {} Codec {} Bandwidth {}",
            self.level, track.name, track.codec, track.bandwidth
        ));

        let cid = track.initialization_segment.link;

        self.state = MachineState::Load;

        spawn_local({
            let cb = self.append_video_cb.clone();
            let ipfs = self.ipfs.clone();

            async move {
                match ipfs.cat(cid, Option::<&str>::None).await {
                    Ok(bytes) => cb.emit(bytes.to_vec()),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });
    }

    /// Append audio and video segments to the buffers.
    fn append_buffers(&self, mut aud_seg: Vec<u8>, mut vid_seg: Vec<u8>) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        if let Err(e) = buffers.audio.append_buffer_with_u8_array(&mut aud_seg) {
            warn!(&format!("{:#?}", e));
        }

        if let Err(e) = buffers.video.append_buffer_with_u8_array(&mut vid_seg) {
            warn!(&format!("{:#?}", e));
        }
    }

    /// Append video segments to the buffer.
    fn append_video_buffer(&self, mut vid_seg: Vec<u8>) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                error!("No Media Buffers");
                return;
            }
        };

        if let Err(e) = buffers.video.append_buffer_with_u8_array(&mut vid_seg) {
            warn!(&format!("{:#?}", e));
        }
    }
}

#![cfg(target_arch = "wasm32")]

use cid::Cid;

use defluencer::{
    channel::{local::LocalUpdater, Channel},
    signatures::ethereum::EthereumSigner,
    user::User,
};

use gloo_console::error;

use utils::defluencer::{ChannelContext, UserContext};

use ybc::{Box, Button, Control, Field, File, Input, Level, LevelItem};

use yew::{platform::spawn_local, prelude::*};

use web_sys::File as SysFile;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct CreateContent {
    video_modal_cb: Callback<MouseEvent>,
    post_modal_cb: Callback<MouseEvent>,
    article_modal_cb: Callback<MouseEvent>,
    modal: Modals,
    close_modal_cb: Callback<MouseEvent>,

    create_cb: Callback<MouseEvent>,

    title: String,
    title_cb: Callback<String>,

    images: Vec<SysFile>,
    image_cb: Callback<Vec<SysFile>>,

    markdowns: Vec<SysFile>,
    makdown_cb: Callback<Vec<SysFile>>,

    video: Cid,
    video_cb: Callback<String>,

    loading: bool,
    disabled: bool,
}

#[derive(PartialEq)]
pub enum Modals {
    None,
    MicroPost,
    Article,
    Video,
}

pub enum Msg {
    Modal(Modals),
    Create,
    CloseModal,
    Title(String),
    Image(Vec<SysFile>),
    Markdown(Vec<SysFile>),
    Video(String),
    Result(Cid),
}

impl Component for CreateContent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let video_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Video));
        let post_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::MicroPost));
        let article_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Article));
        let close_modal_cb = ctx.link().callback(|_| Msg::CloseModal);
        let create_cb = ctx.link().callback(|_| Msg::Create);

        let title_cb = ctx.link().callback(Msg::Title);
        let img_file_cb = ctx.link().callback(Msg::Image);
        let md_file_cb = ctx.link().callback(Msg::Markdown);
        let video_cb = ctx.link().callback(Msg::Video);

        Self {
            video_modal_cb,
            post_modal_cb,
            article_modal_cb,
            modal: Modals::None,
            close_modal_cb,

            title: String::new(),
            title_cb,

            images: vec![],
            image_cb: img_file_cb,

            markdowns: vec![],
            makdown_cb: md_file_cb,

            video: Cid::default(),
            video_cb,

            create_cb,
            loading: false,
            disabled: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Modal(modal) => self.on_modal(modal),
            Msg::CloseModal => self.close_modal(),
            Msg::Create => self.on_create(ctx),
            Msg::Title(title) => self.on_title(title),
            Msg::Image(images) => self.on_images(images),
            Msg::Markdown(markdowns) => self.on_markdowns(markdowns),
            Msg::Video(cid_string) => self.on_video(&cid_string),
            Msg::Result(cid) => self.on_result(cid),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .is_none()
            || ctx
                .link()
                .context::<ChannelContext>(Callback::noop())
                .is_none()
        {
            return html!();
        }

        html! {
        <Box>
            { self.render_modal() }
            <Level>
                <LevelItem>
                    <Button onclick={self.video_modal_cb.clone()} >
                        { "Video" }
                    </Button>
                </LevelItem>
                <LevelItem>
                    <Button onclick={self.post_modal_cb.clone()} >
                        { "Micro Post" }
                    </Button>
                </LevelItem>
                <LevelItem>
                    <Button onclick={self.article_modal_cb.clone()} >
                        { "Article" }
                    </Button>
                </LevelItem>
            </Level>
        </Box>
        }
    }
}

impl CreateContent {
    fn render_modal(&self) -> Html {
        let modal_card_body = match self.modal {
            Modals::MicroPost => html! {
            <section class="modal-card-body">
                <Field label="Text" >
                    <Control>
                        <Input name="text" value="" update={self.title_cb.clone()} />
                    </Control>
                </Field>
            </section>
                },
            Modals::Article => html! {
            <section class="modal-card-body">
                <Field label="Title" >
                    <Control>
                        <Input name="title" value="" update={self.title_cb.clone()} />
                    </Control>
                </Field>
                <Field label="Image File" >
                    <Control>
                        <File name="image" files={self.images.clone()} update={self.image_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                    </Control>
                </Field>
                <Field label="Markdown File" >
                    <Control>
                        <File name="markdown" files={self.markdowns.clone()} update={self.makdown_cb.clone()} selector_label={"Choose a file..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("markdown.md")} fullwidth=true />
                    </Control>
                </Field>
            </section>
                },
            Modals::Video => html! {
            <section class="modal-card-body">
                <Field label="Title" >
                    <Control>
                        <Input name="title" value="" update={self.title_cb.clone()} />
                    </Control>
                </Field>
                <Field label="Processed Video CID" >
                    <Control>
                        <Input name="video_cid" value="" update={self.video_cb.clone()} />
                    </Control>
                </Field>
            </section>
            },
            Modals::None => html! {},
        };

        html! {
        <div class={if self.modal != Modals::None {"modal is-active"} else {"modal"}} >
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "New Content" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                { modal_card_body }
                <footer class="modal-card-foot">
                    <Button onclick={self.create_cb.clone()} loading={self.loading} disabled={self.disabled} >
                        { "Create New Content" }
                    </Button>
                    <Button onclick={self.close_modal_cb.clone()}>
                        { "Cancel" }
                    </Button>
                </footer>
            </div>
        </div>
        }
    }

    fn on_create(&mut self, ctx: &Context<Self>) -> bool {
        let (context, _) = ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .expect("User Context");

        let user = context.user;

        let (context, _) = ctx
            .link()
            .context::<ChannelContext>(Callback::noop())
            .expect("Channel Context");

        let channel = context.channel;

        match self.modal {
            Modals::MicroPost => {
                spawn_local(create_micro_post(
                    user,
                    channel,
                    self.title.clone(),
                    ctx.link().callback(Msg::Result),
                ));
            }
            Modals::Article => spawn_local(create_article(
                user,
                channel,
                self.title.clone(),
                self.images.pop().unwrap(),
                self.markdowns.pop().unwrap(),
                ctx.link().callback(Msg::Result),
            )),
            Modals::Video => spawn_local(create_video_post(
                user,
                channel,
                self.title.clone(),
                self.video,
                self.images.pop().unwrap(),
                ctx.link().callback(Msg::Result),
            )),
            Modals::None => return false,
        }

        self.loading = true;

        true
    }

    fn on_title(&mut self, title: String) -> bool {
        if title.is_empty() {
            self.disabled = true;
        } else {
            self.title = title;
        }

        true
    }

    fn on_video(&mut self, cid_str: &str) -> bool {
        self.video = match Cid::try_from(cid_str) {
            Ok(cid) => cid,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return false;
            }
        };

        true
    }

    fn on_modal(&mut self, modal: Modals) -> bool {
        self.loading = false;
        self.disabled = false;

        self.modal = modal;

        true
    }

    fn close_modal(&mut self) -> bool {
        self.loading = false;
        self.disabled = false;

        self.modal = Modals::None;

        true
    }

    fn on_result(&mut self, _cid: Cid) -> bool {
        self.loading = false;
        self.modal = Modals::None;

        true
    }

    fn on_images(&mut self, images: Vec<SysFile>) -> bool {
        if images.is_empty() {
            self.disabled = true;
        } else {
            self.images = images;
        }

        true
    }

    fn on_markdowns(&mut self, markdowns: Vec<SysFile>) -> bool {
        if markdowns.is_empty() {
            self.disabled = true;
        } else {
            self.markdowns = markdowns;
        }

        true
    }
}

async fn create_micro_post(
    user: User<EthereumSigner>,
    channel: Channel<LocalUpdater>,
    text: String,
    callback: Callback<Cid>,
) {
    let cid = match user.create_micro_blog_post(text).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    match channel.add_content(cid).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn create_video_post(
    user: User<EthereumSigner>,
    channel: Channel<LocalUpdater>,
    title: String,
    cid: Cid,
    image: SysFile,
    callback: Callback<Cid>,
) {
    let cid = match user.create_video_post(title, cid, image).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    match channel.add_content(cid).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn create_article(
    user: User<EthereumSigner>,
    channel: Channel<LocalUpdater>,
    title: String,
    image: SysFile,
    markdown: SysFile,
    callback: Callback<Cid>,
) {
    let cid = match user.create_blog_post(title, image, markdown).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    match channel.add_content(cid).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

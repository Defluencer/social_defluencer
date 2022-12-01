#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use cid::Cid;

use defluencer::{
    channel::{local::LocalUpdater, Channel},
    crypto::signers::MetamaskSigner,
    user::User,
};

use futures_util::{stream::FuturesUnordered, StreamExt};

use gloo_console::error;

use ipfs_api::IpfsService;

use linked_data::types::IPNSAddress;

use utils::{
    defluencer::{ChannelContext, UserContext},
    ipfs::IPFSContext,
};

use ybc::{Button, Buttons, Control, Field, File, Input, Tag, Tags, TextArea};

use yew::{platform::spawn_local, prelude::*};

use web_sys::File as SysFile;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Channel Address
    pub addr: IPNSAddress,
}

pub struct ManageContent {
    video_modal_cb: Callback<MouseEvent>,
    post_modal_cb: Callback<MouseEvent>,
    article_modal_cb: Callback<MouseEvent>,
    follow_modal_cb: Callback<MouseEvent>,
    close_modal_cb: Callback<MouseEvent>,

    modal: Modals,

    create_cb: Callback<MouseEvent>,

    title: String,
    title_cb: Callback<String>,

    images: Vec<SysFile>,
    image_cb: Callback<Vec<SysFile>>,

    markdowns: Vec<SysFile>,
    makdown_cb: Callback<Vec<SysFile>>,

    form_cid: Cid,
    form_cid_cb: Callback<String>,

    form_ipns: IPNSAddress,
    form_ipns_cb: Callback<String>,

    word_count: u64,
    word_cb: Callback<String>,

    tags: HashSet<String>,
    tag_cb: Callback<String>,

    remove_modal_cb: Callback<MouseEvent>,

    loading: bool,
    disabled: bool,
}

#[derive(PartialEq)]
pub enum Modals {
    None,
    MicroPost,
    Article,
    Follow,
    Video,
    Remove,
}

pub enum Msg {
    Modal(Modals),
    Create,
    CloseModal,
    Title(String),
    Image(Vec<SysFile>),
    Markdown(Vec<SysFile>),
    FormCid(String),
    FormIPNS(String),
    WordCount(String),
    Tag(String),
    RemoveTag(String),
    Result(Cid),
}

impl Component for ManageContent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let video_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Video));
        let post_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::MicroPost));
        let article_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Article));
        let follow_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Follow));
        let close_modal_cb = ctx.link().callback(|_| Msg::CloseModal);
        let create_cb = ctx.link().callback(|_| Msg::Create);
        let remove_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Remove));

        let title_cb = ctx.link().callback(Msg::Title);
        let img_file_cb = ctx.link().callback(Msg::Image);
        let md_file_cb = ctx.link().callback(Msg::Markdown);
        let form_cid_cb = ctx.link().callback(Msg::FormCid);
        let form_ipns_cb = ctx.link().callback(Msg::FormIPNS);
        let word_cb = ctx.link().callback(Msg::WordCount);
        let tag_cb = ctx.link().callback(Msg::Tag);

        Self {
            video_modal_cb,
            post_modal_cb,
            article_modal_cb,
            follow_modal_cb,
            modal: Modals::None,
            close_modal_cb,

            title: String::new(),
            title_cb,

            images: vec![],
            image_cb: img_file_cb,

            markdowns: vec![],
            makdown_cb: md_file_cb,

            form_cid: Cid::default(),
            form_cid_cb,

            form_ipns: IPNSAddress::default(),
            form_ipns_cb,

            word_count: 0,
            word_cb,

            tags: Default::default(),
            tag_cb,

            create_cb,

            remove_modal_cb,

            loading: false,
            disabled: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Modal(modal) => self.on_modal(modal),
            Msg::CloseModal => self.close_modal(),
            Msg::Create => self.on_manage(ctx),
            Msg::Title(title) => self.on_title(title),
            Msg::Image(images) => self.on_images(images),
            Msg::Markdown(markdowns) => self.on_markdowns(markdowns),
            Msg::FormCid(cid) => self.on_form_cid(&cid),
            Msg::FormIPNS(addr) => self.on_form_ipns(&addr),
            Msg::WordCount(count) => self.on_word_count(count),
            Msg::Tag(tag) => self.on_tag(tag),
            Msg::RemoveTag(tag) => self.tags.remove(&tag),
            Msg::Result(_) => self.on_result(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <>
        { self.render_modal(ctx) }
        <Buttons classes={classes!("are-normal", "is-centered", "has-addons")}>
            <Button  onclick={self.article_modal_cb.clone()} >
                <span class="icon-text">
                    <span class="icon"><i class="fa-solid fa-plus"></i></span>
                    <span> { "Article" } </span>
                </span>
            </Button>
            <Button  onclick={self.video_modal_cb.clone()} >
                <span class="icon-text">
                    <span class="icon"><i class="fa-solid fa-plus"></i></span>
                    <span> { "Video" } </span>
                </span>
            </Button>
            <Button  onclick={self.post_modal_cb.clone()} >
                <span class="icon-text">
                    <span class="icon"><i class="fa-solid fa-plus"></i></span>
                    <span> { "Comment" } </span>
                </span>
            </Button>
            <Button  onclick={self.follow_modal_cb.clone()} >
                <span class="icon-text">
                    <span class="icon"><i class="fa-solid fa-plus"></i></span>
                    <span> { "Social Web" } </span>
                </span>
            </Button>
            <Button  onclick={self.remove_modal_cb.clone()} >
                <span class="icon-text">
                    <span class="icon"><i class="fa-solid fa-minus"></i></span>
                    <span> { "Remove" } </span>
                </span>
            </Button>
        </Buttons>
        </>
        }
    }
}

impl ManageContent {
    fn render_modal(&self, ctx: &Context<Self>) -> Html {
        let modal_card_body = match self.modal {
            Modals::MicroPost => html! {
            <section class="modal-card-body">
                <Field label="Text" >
                    <Control>
                        <TextArea name="text" value="" update={self.title_cb.clone()} placeholder={"Text here..."} rows={4} fixed_size={true} />
                    </Control>
                </Field>
                {self.render_tags(ctx)}
            </section>
                },
            Modals::Article => html! {
            <section class="modal-card-body">
                <Field label="Title" >
                    <Control>
                        <Input name="title" value="" update={self.title_cb.clone()} />
                    </Control>
                </Field>
                <Field label="Markdown File" >
                    <Control>
                        <File name="markdown" files={self.markdowns.clone()} update={self.makdown_cb.clone()} selector_label={"Choose a file..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("markdown.md")} fullwidth=true />
                    </Control>
                </Field>
                <Field label="Thumbnail Image" help={"Less than 1MiB, 16 by 9 ratio, .PNG or .JPG (optional)"} >
                    <Control>
                        <File name="image" files={self.images.clone()} update={self.image_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                    </Control>
                </Field>
                <Field label="Word Count" help={"Total number of words. (optional)"} >
                    <Control>
                        <Input name="word_count" value="" update={self.word_cb.clone()} />
                    </Control>
                </Field>
                {self.render_tags(ctx)}
            </section>
                },
            Modals::Video => html! {
            <section class="modal-card-body">
                <Field label="Title" >
                    <Control>
                        <Input name="title" value="" update={self.title_cb.clone()} />
                    </Control>
                </Field>
                <Field label="Processed Video CID" help={"Use the Defluencer CLI to transcode your videos."} >
                    <Control>
                        <Input name="video_cid" value="" update={self.form_cid_cb.clone()} />
                    </Control>
                </Field>
                <Field label="Thumbnail Image" help={"Less than 1MiB, 16 by 9 ratio, .PNG or .JPG (optional)"} >
                    <Control>
                        <File name="image" files={self.images.clone()} update={self.image_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                    </Control>
                </Field>
                {self.render_tags(ctx)}
            </section>
            },
            Modals::Follow => html! {
            <section class="modal-card-body">
                <Field label="Channel IPNS Address" help={"Channels can build a social web by adding followees."} >
                    <Control>
                        <Input name="cid" value="" update={self.form_ipns_cb.clone()} />
                    </Control>
                </Field>
            </section>
            },
            Modals::Remove => html! {
            <section class="modal-card-body">
                <Field label="CID" help={"Remove content or followees from your channel."} >
                    <Control>
                        <Input name="cid" value="" update={self.form_cid_cb.clone()} />
                    </Control>
                </Field>
            </section>
            },
            Modals::None => html! {},
        };

        html! {
        <div class={if self.modal != Modals::None {"modal is-active"} else {"modal"}} >
            <div class="modal-background" onclick={self.close_modal_cb.clone()} ></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Content" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                { modal_card_body }
                <footer class="modal-card-foot">
                    <Button onclick={self.create_cb.clone()} loading={self.loading} disabled={self.disabled} >
                        if self.modal != Modals::Remove { {"Create"} } else { {"Remove"} }
                    </Button>
                    <Button onclick={self.close_modal_cb.clone()}>
                        { "Cancel" }
                    </Button>
                </footer>
            </div>
        </div>
        }
    }

    fn render_tags(&self, ctx: &Context<Self>) -> Html {
        let content = self
            .tags
            .iter()
            .map(|tag| {
                let rev_tag = tag.clone();
                //WTF is this double clone why?
                let onclick = ctx.link().callback(move |_: MouseEvent| {
                    let tag = rev_tag.clone();

                    Msg::RemoveTag(tag)
                });

                html! {
                <Control>
                    <Tags has_addons=true >
                        <Tag>
                            {tag.as_str()}
                        </Tag>
                        <Tag {onclick} delete=true />
                    </Tags>
                </Control>
                }
            })
            .collect::<Html>();

        html! {
        <>
        <Field label={"Tags"} help={"Adding a tag publish your content to the pubsub channel of the same name."} >
            <Control>
                <Input name="tagger" value="" update={self.tag_cb.clone()} />
            </Control>
        </Field>
        <Field grouped=true multiline=true >
            {content}
        </Field>
        </>
        }
    }

    fn on_manage(&mut self, ctx: &Context<Self>) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        let user = match ctx.link().context::<UserContext>(Callback::noop()) {
            Some((context, _)) => context.user,
            None => return false,
        };

        let channel = match ctx.link().context::<ChannelContext>(Callback::noop()) {
            Some((context, _)) => context.channel,
            None => return false,
        };

        match self.modal {
            Modals::MicroPost => {
                spawn_local(create_micro_post(
                    ipfs,
                    user,
                    channel,
                    self.title.clone(),
                    self.tags.clone(),
                    ctx.link().callback(Msg::Result),
                ));
            }
            Modals::Article => spawn_local(create_article(
                ipfs,
                user,
                channel,
                self.title.clone(),
                self.images.pop(),
                self.markdowns.pop(),
                self.word_count,
                self.tags.clone(),
                ctx.link().callback(Msg::Result),
            )),
            Modals::Video => spawn_local(create_video_post(
                ipfs,
                user,
                channel,
                self.title.clone(),
                self.form_cid,
                self.images.pop(),
                self.tags.clone(),
                ctx.link().callback(Msg::Result),
            )),
            Modals::Remove => match IPNSAddress::try_from(self.form_cid) {
                Ok(addr) => spawn_local(remove_follow(
                    channel,
                    addr,
                    ctx.link().callback(Msg::Result),
                )),
                Err(_) => spawn_local(remove_content(
                    channel,
                    self.form_cid,
                    ctx.link().callback(Msg::Result),
                )),
            },
            Modals::Follow => spawn_local(add_follow(
                channel,
                self.form_ipns.clone(),
                ctx.link().callback(Msg::Result),
            )),
            Modals::None => return false,
        }

        self.loading = true;

        true
    }

    fn on_modal(&mut self, modal: Modals) -> bool {
        self.loading = false;
        self.disabled = true;

        self.modal = modal;

        //should reset all data?
        self.tags.clear();

        true
    }

    fn close_modal(&mut self) -> bool {
        self.loading = false;
        self.disabled = false;

        self.modal = Modals::None;

        true
    }

    fn on_title(&mut self, title: String) -> bool {
        if title.is_empty() {
            self.disabled = true;
        } else {
            self.title = title;
            self.disabled = false;
        }

        true
    }

    fn on_form_cid(&mut self, cid_str: &str) -> bool {
        self.form_cid = match Cid::try_from(cid_str) {
            Ok(cid) => cid,
            Err(e) => {
                error!(&format!("{:#?}", e));
                self.disabled = true;
                return true;
            }
        };

        self.disabled = false;
        true
    }

    fn on_form_ipns(&mut self, str: &str) -> bool {
        self.form_ipns = match IPNSAddress::try_from(str) {
            Ok(cid) => cid,
            Err(e) => {
                error!(&format!("{:#?}", e));
                self.disabled = true;
                return true;
            }
        };

        self.disabled = false;
        true
    }

    fn on_images(&mut self, images: Vec<SysFile>) -> bool {
        if images.is_empty() {
            self.disabled = true;
        } else {
            self.images = images;
            self.disabled = false;
        }

        true
    }

    fn on_markdowns(&mut self, markdowns: Vec<SysFile>) -> bool {
        if markdowns.is_empty() {
            self.disabled = true;
        } else {
            self.markdowns = markdowns;
            self.disabled = false;
        }

        true
    }

    fn on_result(&mut self) -> bool {
        if let Modals::None = self.modal {
            return false;
        }

        self.loading = false;
        self.modal = Modals::None;

        true
    }

    fn on_word_count(&mut self, count: String) -> bool {
        if count.is_empty() {
            self.disabled = true;
            return true;
        }

        self.word_count = match count.parse::<u64>() {
            Ok(count) => count,
            Err(e) => {
                error!(&format!("{:#?}", e));
                self.disabled = true;
                return true;
            }
        };

        self.disabled = false;

        true
    }

    fn on_tag(&mut self, tag: String) -> bool {
        if !tag.ends_with(' ') {
            return false;
        }

        let tags: Vec<&str> = tag.split(' ').collect();

        let mut update = false;

        for tag in tags {
            if tag.is_empty() || tag == " " {
                continue;
            }

            if self.tags.insert(tag.to_owned()) {
                update = true;
            }
        }

        update
    }
}

async fn create_micro_post(
    ipfs: IpfsService,
    user: User<MetamaskSigner>,
    channel: Channel<LocalUpdater>,
    text: String,
    tags: HashSet<String>,
    callback: Callback<Cid>,
) {
    let cid = match user.create_micro_blog_post(text, None, false).await {
        Ok((cid, _)) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    publish_tags(ipfs, cid, tags).await;

    match channel.add_content(cid).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn create_video_post(
    ipfs: IpfsService,
    user: User<MetamaskSigner>,
    channel: Channel<LocalUpdater>,
    title: String,
    cid: Cid,
    image: Option<SysFile>,
    tags: HashSet<String>,
    callback: Callback<Cid>,
) {
    let cid = match user.create_video_post(title, cid, image, false).await {
        Ok((cid, _)) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    publish_tags(ipfs, cid, tags).await;

    match channel.add_content(cid).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn create_article(
    ipfs: IpfsService,
    user: User<MetamaskSigner>,
    channel: Channel<LocalUpdater>,
    title: String,
    image: Option<SysFile>,
    markdown: Option<SysFile>,
    count: u64,
    tags: HashSet<String>,
    callback: Callback<Cid>,
) {
    let markdown = match markdown {
        Some(md) => md,
        None => return,
    };

    let count = if count == 0 { None } else { Some(count) };

    let cid = match user
        .create_blog_post(title, image, markdown, count, false)
        .await
    {
        Ok((cid, _)) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    publish_tags(ipfs, cid, tags).await;

    match channel.add_content(cid).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn publish_tags(ipfs: IpfsService, cid: Cid, tags: HashSet<String>) {
    let mut pub_pool: FuturesUnordered<_> = tags
        .into_iter()
        .map(|topic| ipfs.pubsub_pub(topic, cid.to_bytes()))
        .collect();

    while let Some(result) = pub_pool.next().await {
        if let Err(e) = result {
            error!(&format!("{:#?}", e));
        }
    }
}

async fn remove_content(channel: Channel<LocalUpdater>, cid: Cid, callback: Callback<Cid>) {
    match channel.remove_content(cid).await {
        Ok(option) => match option {
            Some(cid) => callback.emit(cid),
            None => error!("Cannot find content to remove!"),
        },
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn add_follow(channel: Channel<LocalUpdater>, addr: IPNSAddress, callback: Callback<Cid>) {
    match channel.follow(addr).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn remove_follow(channel: Channel<LocalUpdater>, addr: IPNSAddress, callback: Callback<Cid>) {
    match channel.unfollow(addr).await {
        Ok(cid) => callback.emit(cid),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

#![cfg(target_arch = "wasm32")]

use cid::Cid;

use defluencer::{
    channel::{local::LocalUpdater, Channel},
    crypto::signers::EthereumSigner,
    user::User,
};

use gloo_console::error;

use utils::defluencer::{ChannelContext, UserContext};

use ybc::{Box, Button, Control, Field, File, Input, Level, LevelItem, TextArea};

use yew::{platform::spawn_local, prelude::*};

use web_sys::File as SysFile;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Channel Address
    pub addr: Cid,
}

pub struct ManageContent {
    video_modal_cb: Callback<MouseEvent>,
    post_modal_cb: Callback<MouseEvent>,
    article_modal_cb: Callback<MouseEvent>,
    modal: Modals,
    close_modal_cb: Callback<MouseEvent>,

    manage_cb: Callback<MouseEvent>,

    title: String,
    title_cb: Callback<String>,

    images: Vec<SysFile>,
    image_cb: Callback<Vec<SysFile>>,

    markdowns: Vec<SysFile>,
    makdown_cb: Callback<Vec<SysFile>>,

    form_cid: Cid,
    form_cid_cb: Callback<String>,

    remove_modal_cb: Callback<MouseEvent>,

    loading: bool,
    disabled: bool,
}

#[derive(PartialEq)]
pub enum Modals {
    None,
    MicroPost,
    Article,
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
    Result(Cid),
}

impl Component for ManageContent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let video_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Video));
        let post_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::MicroPost));
        let article_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Article));
        let close_modal_cb = ctx.link().callback(|_| Msg::CloseModal);
        let create_cb = ctx.link().callback(|_| Msg::Create);
        let remove_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Remove));

        let title_cb = ctx.link().callback(Msg::Title);
        let img_file_cb = ctx.link().callback(Msg::Image);
        let md_file_cb = ctx.link().callback(Msg::Markdown);
        let form_cid_cb = ctx.link().callback(Msg::FormCid);

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

            form_cid: Cid::default(),
            form_cid_cb,

            manage_cb: create_cb,

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
            Msg::FormCid(cid_string) => self.on_form_cid(&cid_string),
            Msg::Result(_) => self.on_result(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
        <Box>
            { self.render_modal() }
            <Level>
                <LevelItem>
                    <Button onclick={self.video_modal_cb.clone()} >
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-plus"></i></span>
                            <span> { "Video" } </span>
                        </span>
                    </Button>
                </LevelItem>
                <LevelItem>
                    <Button onclick={self.post_modal_cb.clone()} >
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-plus"></i></span>
                            <span> { "Micro Post" } </span>
                        </span>
                    </Button>
                </LevelItem>
                <LevelItem>
                    <Button onclick={self.article_modal_cb.clone()} >
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-plus"></i></span>
                            <span> { "Article" } </span>
                        </span>
                    </Button>
                </LevelItem>
                <LevelItem>
                    <Button onclick={self.remove_modal_cb.clone()} >
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-minus"></i></span>
                            <span> { "Remove" } </span>
                        </span>
                    </Button>
                </LevelItem>
            </Level>
        </Box>
        }
    }
}

impl ManageContent {
    fn render_modal(&self) -> Html {
        let modal_card_body = match self.modal {
            Modals::MicroPost => html! {
            <section class="modal-card-body">
                <Field label="Text" >
                    <Control>
                        <TextArea name="text" value="" update={self.title_cb.clone()} placeholder={"Text here..."} rows={4} fixed_size={true} />
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
                        <Input name="video_cid" value="" update={self.form_cid_cb.clone()} />
                    </Control>
                </Field>
                <Field label="Image File" >
                    <Control>
                        <File name="image" files={self.images.clone()} update={self.image_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                    </Control>
                </Field>
            </section>
            },
            Modals::Remove => html! {
            <section class="modal-card-body">
                <Field label="Content CID" >
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
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Manage Content" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                { modal_card_body }
                <footer class="modal-card-foot">
                    <Button onclick={self.manage_cb.clone()} loading={self.loading} disabled={self.disabled} >
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

    fn on_manage(&mut self, ctx: &Context<Self>) -> bool {
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
                self.form_cid,
                self.images.pop().unwrap(),
                ctx.link().callback(Msg::Result),
            )),
            Modals::Remove => spawn_local(remove_content(
                channel,
                self.form_cid,
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
                return false;
            }
        };

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
}

async fn create_micro_post(
    user: User<EthereumSigner>,
    channel: Channel<LocalUpdater>,
    text: String,
    callback: Callback<Cid>,
) {
    let cid = match user.create_micro_blog_post(text, false).await {
        Ok((cid, _)) => cid,
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
    let cid = match user.create_video_post(title, cid, image, false).await {
        Ok((cid, _)) => cid,
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
    let cid = match user.create_blog_post(title, image, markdown, false).await {
        Ok((cid, _)) => cid,
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

async fn remove_content(channel: Channel<LocalUpdater>, cid: Cid, callback: Callback<Cid>) {
    match channel.remove_content(cid).await {
        Ok(option) => match option {
            Some(cid) => callback.emit(cid),
            None => {}
        },
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

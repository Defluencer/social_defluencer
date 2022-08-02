use ybc::{Box, Button, Container, Content, Level, LevelItem, LevelLeft, Section, Subtitle};

use yew::prelude::*;

use gloo_console::{error, info};

use gloo_storage::{LocalStorage, Storage};

use linked_data::identity::Identity;

const CURRENT_ID_KEY: &str = "current_id";
const ID_LIST_KEY: &str = "id_list";

#[derive(Properties, PartialEq)]
pub struct Props {}

pub struct IdentitySettings {
    pub modal: bool,
    pub modal_cb: Callback<MouseEvent>,

    current_id: usize,
    id_list: Vec<Identity>,
}

pub enum Msg {
    Modal,
    SetID(usize),
}

impl Component for IdentitySettings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Identity Setting Create");

        let current_id: usize = LocalStorage::get(CURRENT_ID_KEY).unwrap_or_default();
        let id_list: Vec<Identity> = LocalStorage::get(ID_LIST_KEY).unwrap_or_default();

        Self {
            modal: false,
            modal_cb: ctx.link().callback(|_| Msg::Modal),

            current_id,
            id_list,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Identity Setting Update");

        match msg {
            Msg::Modal => {
                self.modal = !self.modal;

                true
            }
            Msg::SetID(i) => {
                if let Err(e) = LocalStorage::set(CURRENT_ID_KEY, i) {
                    error!(&format!("{:?}", e));
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Identity Setting View");

        html! {
        <Section>
            <Container>
                <Subtitle >
                    { "Identities" }
                </Subtitle>
                { self.render_modal() }
                {
                    self.id_list.iter().enumerate().map(|(i, item)| {
                        let disabled = i == self.current_id;

                        let cb = ctx.link().callback(move |_: MouseEvent| { Msg::SetID(i) });

                        self.render_id(disabled, item, cb)

                    } ).collect::<Html>()
                }
                <Button onclick={ self.modal_cb.clone() } >
                    { "Create New Identity" }
                </Button>
            </Container>
        </Section>
        }
    }
}

impl IdentitySettings {
    fn render_modal(&self) -> Html {
        html! {
        <div class= { if self.modal { "modal is-active" } else { "modal" } } >
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "New Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                    <div class="field">
                        <label class="label"> { "Display Name" } </label>
                        <div class="control is-expanded">
                            <input class="input" type="text" name="name_input" />
                        </div>
                        <p class="help"> { "Refresh to apply changes." } </p>
                    </div>
                    //TODO create an Identity from the form data
                </section>
                <footer class="modal-card-foot">
                    <button class="button is-success" onclick={/*TODO add Identity to IPFS*/}>
                        { "Save changes" }
                    </button>
                    <button class="button" onclick={self.modal_cb.clone()}>
                        { "Cancel" }
                    </button>
                </footer>
            </div>
        </div>
        }
    }

    fn render_id(
        &self,
        disabled: bool,
        identity: &Identity,
        onclick: Callback<MouseEvent>,
    ) -> Html {
        html! {
        <Level>
            <LevelLeft>
                <LevelItem>
                    <Button {disabled} {onclick} >
                        { "Set" }
                    </Button>
                </LevelItem>
                <LevelItem>
                    { &identity.display_name }
                </LevelItem>
            </LevelLeft>
        </Level>
        }
    }
}

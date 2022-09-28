#![cfg(target_arch = "wasm32")]

use ybc::{Box, Column, Columns, Section};

use yew::{classes, function_component, html, Html, Properties};

use cid::Cid;

use components::{chat::ChatWindow, pure::NavigationBar, video_player::VideoPlayer};

#[derive(Properties, PartialEq)]
pub struct LivePageProps {
    pub cid: Cid,
}

/// social.defluencer.eth/#/live/CID_HERE
#[function_component(LivePage)]
pub fn live_page(props: &LivePageProps) -> Html {
    html! {
    <>
    <NavigationBar />
    <Section>
        <Columns>
            <Column>
                <Box>
                    <VideoPlayer cid={props.cid} />
                </Box>
            </Column>
            <Column classes={classes!("is-one-fifth")} >
                <ChatWindow cid={props.cid} />
            </Column>
        </Columns>
    </Section>
    </>
    }
}

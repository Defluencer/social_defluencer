#![cfg(target_arch = "wasm32")]

use components::{pure::NavigationBar, Route};

use ybc::{
    ButtonRouter, Card, CardContent, Column, Columns, Container, Content, Footer, HeaderSize, Hero,
    Media, MediaContent, Section, Subtitle, Title,
};
use yew::{classes, function_component, html, Html};

// Header This is decentralized social media
// Features explanations
// Button to start -> config IPFS
// Footer github, gitcoin, etc...

/// social.defluencer.eth/#/home/
/// The App Landing Page.
#[function_component(HomePage)]
pub fn home_page() -> Html {
    let body_one = html! {
        <Container classes={classes!("has-text-centered") }>
            <Title tag="h1" >
            {
                "social.defluencer.eth is the first social media app built on the Defluencer protocol."
            }
            </Title>
             <br />
            <Subtitle tag="h2" >
            {"Defluencer inherit all the good properties of IPFS."}
            <br/>
            {"Local first means it will work on local networks not connected to the internet or when connectivity is loss."}
            <br/>
            {"Every users, channels and media content can be used by any other current or future apps on the protocol."}
            <br/>
            {"Unlike blockchains this protocol is logically decentralized and scale very well."}
            </Subtitle>
            <ButtonRouter<Route> route={Route::Settings} classes={classes!("is-primary")} >
                {"Get Started"}
            </ButtonRouter<Route>>
        </Container>
    };

    let live_card = feature_card(
        "Live Streaming",
        "Set custom resolution, quality and codecs. (experimental)",
    );

    let chat_card = feature_card(
        "Live Chat",
        "Exchange instant messages with other people online. (experimental)",
    );

    let streaming_card = feature_card(
        "On Demand Streaming",
        "Host your own videos or view past live streams.",
    );

    let blog_card = feature_card(
        "Blogs",
        "Twitter-style micro blog posts or lengthy articles.",
    );

    let feed_card = feature_card(
        "Content Feed",
        "Organize a multimedia feed of the channels you follow.",
    );

    let comments_card = feature_card(
        "Commentary",
        "Comment on any media and read what people you follow have to say.",
    );

    html! {
        <>
            <NavigationBar />
            <Hero classes={classes!("is-medium")} body={body_one} />
            <Section>
                <Container>
                    <Columns classes={classes!("is-multiline")} >
                        { live_card }
                        { chat_card }
                        { streaming_card }
                        { blog_card }
                        { feed_card }
                        { comments_card }
                    </Columns>
                </Container>
            </Section>
            <Section>
                <Container>
                    <Content>
                        <p>
                            <Title size={HeaderSize::Is5} >
                                { "How does the protocol work?" }
                            </Title>
                            {
                                "Defluencer is a protocol built on top of the inter-planetary file system (IPFS).
                                On IPFS, data is content addressed which means your content can be shared but never modified.
                                As content go viral, it is replicated by anyone who reads, watches or interacts with it in any way, resulting in social media without central servers."
                            }
                        </p>
                        <p>
                            {"Social media content is cryptographically signed. By doing so, each piece of content becomes verifiable."}
                        </p>
                        <p>
                            {"Websites or applications folowing the protocol become interoperable with each other because of these properties."}
                        </p>
                        <p>
                            <Title size={HeaderSize::Is5} >
                                { "What can be built on top of the protocol?" }
                            </Title>
                            {"There are many things, here's some ideas."}
                            <ul>
                                <li>{"Personal blogs."}</li>
                                <li>{"Community forums."}</li>
                                <li>{"Topical content aggregators."}</li>
                                <li>{"Blockchain content timestamping services."}</li>
                                <li>{"Defluencer specific IPFS gateways."}</li>
                                <li>{"Content Moderation and filtering services."}</li>
                                <li>{"Pinning and Archiving services."}</li>
                            </ul>
                        </p>
                    </Content>
                </Container>
            </Section>
            <Footer>
                <Container>
                    <Columns>
                        <Column classes={classes!("is-half")} >
                            <a href="https://github.com/Defluencer">
                                <span class="icon-text">
                                    <span> {"Source Code"} </span>
                                    <span class="icon"><i class="fab fa-github"></i></span>
                                </span>
                            </a>
                        </Column>
                        <Column classes={classes!("is-half")} >
                            <a href="https://bulma.io">
                                <img src="https://bulma.io/images/made-with-bulma.png" alt="Made with Bulma" width="128" height="24" />
                            </a>
                        </Column>
                    </Columns>
                </Container>
            </Footer>
        </>
    }
}

fn feature_card(title: &str, text: &str) -> Html {
    html! {
        <Column classes={classes!("is-half", "is-flex")} >
            <Card>
                <CardContent>
                    <Media>
                        <MediaContent>
                            <Title tag="h1" size={HeaderSize::Is4 }> { title } </Title>
                        </MediaContent>
                    </Media>
                    <Content>
                        <Subtitle tag="div" > { text } </Subtitle>
                    </Content>
                </CardContent>
            </Card>
        </Column>
    }
}

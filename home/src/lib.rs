#![cfg(target_arch = "wasm32")]

use components::{pure::NavigationBar, Route};

use ybc::{
    Block, ButtonRouter, Card, CardContent, Column, Columns, Container, Content, Footer,
    HeaderSize, Hero, Media, MediaContent, Section, Subtitle, Title,
};

use yew::{classes, function_component, html, Html};

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
            {"Every users, channels and media content can be used by any current or future apps on the protocol."}
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
        "Set custom resolution, quality and codecs.",
    );

    let chat_card = feature_card(
        "Live Chat",
        "Exchange instant messages with other people online.",
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
                                { "How to I use this app?" }
                            </Title>
                            {"First, configure IPFS then you can create an identity."}
                            <br/>
                            {"A channel can be created at the same time and is used to aggregate content. This is optional but most people like to have a personnal channel."}
                            <br/>
                            {"You can then publish content, share, comment and live stream but only using the Defluencer CLI."}
                            <br/>
                            {"Find new channels by listing the followees of any channel you like or search ENS. Don't forget to add your favorite channels to your social web."}
                            <br/>
                            {"N.B. Comment aggregation is done by crawling the social web. You will always see the comments of people closer to you first."}
                        </p>
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
                                { "What can be built on the protocol?" }
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
                            <Block>
                                <a href="https://github.com/Defluencer">
                                    <span class="icon-text">
                                        <span class="icon"><i class="fab fa-github"></i></span>
                                        <span> {"Source Code"} </span>
                                    </span>
                                </a>
                            </Block>
                            <Block>
                                <a href="https://ipfs.tech">
                                    <span class="icon-text">
                                        <svg class="icon" viewBox="0 0 235.3 235.3" height="23.53" width="23.53" >
                                            <path d="M30.3 164l84 48.5 84-48.5V67l-84-48.5-84 48.5v97z" fill="#469ea2"></path>
                                            <path d="M105.7 30.1l-61 35.2a18.19 18.19 0 010 3.3l60.9 35.2a14.55 14.55 0 0117.3 0l60.9-35.2a18.19 18.19 0 010-3.3L123 30.1a14.55 14.55 0 01-17.3 0zm84 48.2l-61 35.6a14.73 14.73 0 01-8.6 15l.1 70a15.57 15.57 0 012.8 1.6l60.9-35.2a14.73 14.73 0 018.6-15V79.9a20 20 0 01-2.8-1.6zm-150.8.4a15.57 15.57 0 01-2.8 1.6v70.4a14.38 14.38 0 018.6 15l60.9 35.2a15.57 15.57 0 012.8-1.6v-70.4a14.38 14.38 0 01-8.6-15L38.9 78.7z" fill="#6acad1"></path>
                                            <path d="M114.3 29l75.1 43.4v86.7l-75.1 43.4-75.1-43.4V72.3L114.3 29m0-10.3l-84 48.5v97l84 48.5 84-48.5v-97l-84-48.5z" fill="#469ea2"></path>
                                            <path d="M114.9 132h-1.2A15.66 15.66 0 0198 116.3v-1.2a15.66 15.66 0 0115.7-15.7h1.2a15.66 15.66 0 0115.7 15.7v1.2a15.66 15.66 0 01-15.7 15.7zm0 64.5h-1.2a15.65 15.65 0 00-13.7 8l14.3 8.2 14.3-8.2a15.65 15.65 0 00-13.7-8zm83.5-48.5h-.6a15.66 15.66 0 00-15.7 15.7v1.2a15.13 15.13 0 002 7.6l14.3-8.3V148zm-14.3-89a15.4 15.4 0 00-2 7.6v1.2a15.66 15.66 0 0015.7 15.7h.6V67.2L184.1 59zm-69.8-40.3L100 26.9a15.73 15.73 0 0013.7 8.1h1.2a15.65 15.65 0 0013.7-8l-14.3-8.3zM44.6 58.9l-14.3 8.3v16.3h.6a15.66 15.66 0 0015.7-15.7v-1.2a16.63 16.63 0 00-2-7.7zM30.9 148h-.6v16.2l14.3 8.3a15.4 15.4 0 002-7.6v-1.2A15.66 15.66 0 0030.9 148z" fill="#469ea2"></path>
                                            <path d="M114.3 213.2v-97.1l-84-48.5v97.1z" fill-opacity=".15" fill="#083b54"></path>
                                            <path d="M198.4 163.8v-97l-84 48.5v97.1z" fill-opacity=".05" fill="#083b54"></path>
                                        </svg>
                                        <span> {"IPFS"} </span>
                                    </span>
                                </a>
                            </Block>
                            <Block>
                                <a href="https://ipld.io">
                                    <span class="icon-text">
                                        <svg class="icon" viewBox="150 70 235.3 235.3" height="23.53" width="23.53" >
                                            <g class="ipld-logo">
                                                <path d="M353.101 229.707v-91.575c0-3.118-1.663-6-4.364-7.559l-79.306-45.787a8.73 8.73 0 0 0-8.729 0l-79.306 45.787a8.728 8.728 0 0 0-4.364 7.56v91.574c0 3.119 1.663 6 4.364 7.56l79.306 45.787a8.73 8.73 0 0 0 8.729 0l79.306-45.788a8.729 8.729 0 0 0 4.364-7.559z" fill="#1D74F2" fill-rule="nonzero"></path>
                                                <g transform="translate(186.222 94.965)" fill-rule="nonzero" fill="#FFF">
                                                    <path d="M78.844 175.25L3.068 130.88V47.492L78.842 2.66l.108.061 75.67 42.983v85.178l-.106.062-75.67 44.308zM3.499 130.634l75.345 44.119 75.345-44.119v-84.68L78.846 3.157 3.5 47.736v82.897z"></path>
                                                    <g transform="translate(0 44.959)">
                                                        <circle cx="3.068" cy="85.922" r="3.027"></circle>
                                                        <circle cx="3.068" cy="3.37" r="3.027"></circle>
                                                    </g>
                                                    <g transform="translate(151.268 44.959)">
                                                        <circle cx="3.352" cy="85.728" r="3.027"></circle>
                                                        <circle cx="3.352" cy="3.177" r="3.027"></circle>
                                                    </g>
                                                    <circle cx="78.844" cy="174.434" r="3.027"></circle>
                                                    <circle cx="78.842" cy="3.475" r="3.027"></circle>
                                                </g>
                                                <g transform="translate(202 117)">
                                                    <path class="ipld-logo-trunk" d="M63 149.5V9" stroke="#FFF" stroke-width="4" stroke-linecap="square"></path>
                                                    <path class="ipld-logo-branch" stroke="#FFF" stroke-width="4" stroke-linecap="square" d="M64.5 112.5L121 77v-9.513M61.5 112.5L6 82v-9.014M61.5 84.5l-40-22V47.492m43 26.008l35-22V43m-72 48.5v-24m34-24l-15-9V22.99M46 73.5v-16m18.5-4l15-9.5V29m12 65.5v-37"></path>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="5.5" cy="69.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="21.5" cy="43.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="45.5" cy="53.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="46.5" cy="19.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="62.5" cy="5.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="79.5" cy="24.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="99.5" cy="39.5" r="5.5"></circle>
                                                    <circle class="ipld-logo-leaf" fill="#FFF" cx="121.5" cy="64.5" r="5.5"></circle>
                                                </g>
                                            </g>
                                        </svg>
                                        <span> {"IPLD"} </span>
                                    </span>
                                </a>
                            </Block>
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

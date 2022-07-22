use yew::{function_component, html};

/// A Channel Page.
#[function_component(ChannelPage)]
pub fn channel() -> Html {
    html! { "Hello world" }
}

// social.defluencer.eth/#/channel/<IPNS_HERE>
// Stream content metadata from channel
// Subscribe to the IPNS pubsub for live updates
// If your channel, add buttons to post & remove stuff
// If live, display video

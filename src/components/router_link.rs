use yew::prelude::*;
use router::Route;
use router::{Router, Request};


#[derive(Default, Clone, Debug, PartialEq)]
pub struct Props {
    route: Route<()>,
    text: String
}

pub enum Msg {
    NoOp,
    Clicked
}

pub struct RouterLink {
    router: Box<Bridge<Router<()>>>,
    route: Route<()>,
    text: String
}


impl Component for RouterLink {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|_route: Route<()>| Msg::NoOp);
        let router = Router::bridge(callback);

        RouterLink {
            router,
            route: props.route,
            text: props.text
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => false,
            Msg::Clicked => {
                self.router.send(Request::ChangeRoute(self.route.clone()));
                false
            }
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.route = props.route;
        self.text = props.text;
        true
    }
}

impl Renderable<RouterLink> for RouterLink {
    fn view(&self) -> Html<RouterLink> {
        html! {
            <a onclick=|_| Msg::Clicked, >{&self.text}</a>
        }
    }
}
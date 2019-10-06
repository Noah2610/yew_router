use yew::prelude::*;
use yew_router::prelude::*;

use crate::page_not_found::PageNotFound;

pub struct BComp {}

pub enum Msg {}

impl Component for BComp {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        BComp {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn destroy(&mut self) {
        log::info!("BComp destroyed")
    }

    fn view(&self) -> Html<Self> {
        html! {
            <>
                <div>
                    { "I am the B component"}
                </div>
                <div>
                    <Router>
                        <Route matcher=route!("/b/{*}") render=component::<PageNotFound>() />
                    </Router>
                </div>
            </>
        }
    }
}

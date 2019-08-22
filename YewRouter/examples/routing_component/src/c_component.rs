use yew::prelude::*;
use yew::Properties;
use yew_router::yew_router_derive::FromMatches;

pub struct CModel;

#[derive(PartialEq, Properties, FromMatches)]
pub struct Props{}

pub enum Msg {
}


impl Component for CModel {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        CModel
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}


impl Renderable<CModel> for CModel {
    fn view(&self) -> Html<Self> {

        html! {
            <div>
                {" I am the C component"}
            </div>
        }
    }
}


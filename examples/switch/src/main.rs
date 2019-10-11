fn main() {
    let route = RouteInfo::<()>::from("/some/route");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/some/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/another/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);


    let route = RouteInfo::<()>::from("/inner/left");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/yeet");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);



    let route = RouteInfo::<()>::from("/single/32");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/othersingle/472");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);
}
use yew_router::route_info::RouteInfo;
use yew_router::Switch;

#[derive(Switch, Debug)]
pub enum AppRoute {
    #[to = "/some/route"]
    SomeRoute,
    #[to = "/some/{thing}"]
    Something { thing: String },
    #[to = "/another/{thing}"]
    Another(String),
    #[to = "/inner{*:inner}"]
    Nested(InnerRoute),
    #[to = "{*:x}"]
    Single(Single),
    #[to = "{*:x}"]
    OtherSingle(OtherSingle)
}

#[derive(Switch, Debug)]
pub enum InnerRoute {
    #[to = "/left"]
    Left,
    #[to = "/right"]
    Right
}

#[derive(Switch, Debug)]
#[to = "/single/{number}"]
pub struct Single {
    number: u32
}

#[derive(Switch, Debug)]
#[to = "/othersingle/{number}"]
pub struct OtherSingle(u32);


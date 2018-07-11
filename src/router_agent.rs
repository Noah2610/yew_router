//! Routing service
use routing_service::RouteService;

use yew::prelude::worker::*;

use std::collections::HashSet;

use serde::Serialize;
use serde::Deserialize;
use std::fmt::Debug;

use route::RouteBase;
use route::RouteState;
use yew::callback::Callback;


/// Any state that can be used in the router agent must meet the criteria of this trait.
pub trait RouterState<'de>: RouteState + Serialize + Deserialize<'de> + Debug {}
impl <'de, T> RouterState<'de> for T
    where T: RouteState + Serialize + Deserialize<'de> + Debug
{}

pub enum Msg<T>
    where T: RouteState
{
    BrowserNavigationRouteChanged((String, T)),
}




#[derive(Serialize, Deserialize, Debug)]
pub enum RouterRequest<T> {
    /// Replaces the most recent Route with a new one and alerts connected components to the route change.
    ReplaceRoute(RouteBase<T>),
    /// Replaces the most recent Route with a new one, but does not alert connected components to the route change.
    ReplaceRouteNoBroadcast(RouteBase<T>),
    /// Changes the route using a Route struct and alerts connected components to the route change.
    ChangeRoute(RouteBase<T>),
    /// Changes the route using a Route struct, but does not alert connected components to the route change.
    ChangeRouteNoBroadcast(RouteBase<T>),
    /// Gets the current route.
    GetCurrentRoute
}

impl <T> Transferable for RouterRequest<T>
    where for <'de> T: Serialize + Deserialize<'de>
{}

/// A simplified routerBase that assumes that no state is stored.
pub type Router = RouterBase<()>;
/// A simplified interface to the router agent
pub struct RouterBase<T>(Box<Bridge<RouterAgentBase<T>>>)
    where for<'de> T: RouterState<'de>;


pub type RouterAgent = RouterAgentBase<()>;

/// The Router agent holds on to the RouteService singleton and mediates access to it.
pub struct RouterAgentBase<T>
    where for<'de> T: RouterState<'de>
{
    link: AgentLink<RouterAgentBase<T>>,
    route_service: RouteService<T>,
    /// A list of all entities connected to the router.
    /// When a route changes, either initiated by the browser or by the app,
    /// the route change will be broadcast to all listening entities.
    subscribers: HashSet<HandlerId>,
}

impl<T> Agent for RouterAgentBase<T>
    where for<'de> T: RouterState<'de>
{
    type Reach = Context;
    type Message = Msg<T>;
    type Input = RouterRequest<T>;
    type Output = RouteBase<T>;

    fn create(link: AgentLink<Self>) -> Self {
        let callback = link.send_back(|route_changed: (String, T)| Msg::BrowserNavigationRouteChanged(route_changed));
        let mut route_service = RouteService::new();
        route_service.register_callback(callback);

        RouterAgentBase {
            link,
            route_service,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::BrowserNavigationRouteChanged((_route_string, state)) => {
                trace!("Browser navigated");
                let mut route = RouteBase::current_route(&self.route_service);
                route.state = state;
                for sub in self.subscribers.iter() {
                    self.link.response(*sub, route.clone());
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            RouterRequest::ReplaceRoute(route) => {
                trace!("Replacing route and broadcasting to {} subscribers", self.subscribers.len());
                let route_string: String = route.to_route_string();
                self.route_service.replace_route(&route_string, route.state);
                let route = RouteBase::current_route(&self.route_service);
                for sub in self.subscribers.iter() {
                    self.link.response(*sub, route.clone());
                }
            }
            RouterRequest::ReplaceRouteNoBroadcast(route) => {
                trace!("Replacing route and not broadcasting");
                let route_string: String = route.to_route_string();
                self.route_service.replace_route(&route_string, route.state);
            }
            RouterRequest::ChangeRoute(route) => {
                trace!("Changing route and broadcasting route to {} subscribers", self.subscribers.len());
                let route_string: String = route.to_route_string();
                // set the route
                self.route_service.set_route(&route_string, route.state);
                // get the new route. This will contain a default state object
                let route = RouteBase::current_route(&self.route_service);
                // broadcast it to all listening components
                for sub in self.subscribers.iter() {
                    self.link.response(*sub, route.clone());
                }
            }
            RouterRequest::ChangeRouteNoBroadcast(route) => {
                trace!("Changing route and not broadcasting");
                let route_string: String = route.to_route_string();
                self.route_service.set_route(&route_string, route.state);
            }
            RouterRequest::GetCurrentRoute => {
                trace!("Getting route");
                let route = RouteBase::current_route(&self.route_service);
                self.link.response(who, route.clone());
            }
        }
    }
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl <T> RouterBase<T>
    where for<'de> T: RouterState<'de>
{
    pub fn new(callback: Callback<RouteBase<T>>) -> Self {
        let router_agent = RouterAgentBase::bridge(callback);
        RouterBase(router_agent)
    }

    /// Experimental, may be removed
    ///
    /// Directly spawn a new Router
    pub fn spawn(callback: Callback<RouteBase<T>>) -> Self {
        use yew::agent::Discoverer;
        let router_agent = Context::spawn_or_join(callback);
        RouterBase(router_agent)
    }

    pub fn send(&mut self, request: RouterRequest<T>) {
        self.0.send(request)
    }
}



/// A sender for the Router that doesn't send messages back to the component that connects to it.
///
/// This may be subject to change
pub struct RouterSenderAgentBase<T>
    where for<'de> T: RouterState<'de>
{
    router_agent: Box<Bridge<RouterAgentBase<T>>>
}

#[derive(Serialize, Deserialize)]
pub struct Void;
impl Transferable for Void {}

impl<T> Agent for RouterSenderAgentBase<T>
    where for<'de> T: RouterState<'de>
{
    type Reach = Context;
    type Message = ();
    type Input = RouterRequest<T>;
    type Output = Void;

    fn create(link: AgentLink<Self>) -> Self {
        RouterSenderAgentBase {
            router_agent: RouterAgentBase::bridge(link.send_back(|_| ()))
        }
    }

    fn update(&mut self, _msg: Self::Message) {
    }

    fn handle(&mut self, msg: Self::Input, _who: HandlerId) {
        self.router_agent.send(msg);
    }

}

pub type RouterSender = RouterSenderBase<()>;

/// A simplified interface to the router agent
pub struct RouterSenderBase<T>(Box<Bridge<RouterSenderAgentBase<T>>>)
    where for<'de> T: RouterState<'de>;

impl <T> RouterSenderBase<T>
    where for<'de> T: RouterState<'de>
{
    pub fn new(callback: Callback<Void>) -> Self {
        let router_agent = RouterSenderAgentBase::bridge(callback);
        RouterSenderBase(router_agent)
    }

    pub fn send(&mut self, request: RouterRequest<T>) {
        self.0.send(request)
    }
}

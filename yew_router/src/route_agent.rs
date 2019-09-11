//! Routing service
use crate::route_service::RouteService;

use yew::prelude::worker::*;

use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Debug, Formatter, Error as FmtError};

use crate::route_info::RouteInfo;
use crate::route_info::RouteState;
use yew::callback::Callback;
use log::trace;
use std::ops::{Deref, DerefMut};

/// Any state that can be used in the router agent must meet the criteria of this trait.
pub trait RouterState<'de>: RouteState + Serialize + Deserialize<'de> + Debug {}
impl<'de, T> RouterState<'de> for T where T: RouteState + Serialize + Deserialize<'de> + Debug {}

/// Message used for the RouteAgent.
#[derive(Debug)]
pub enum Msg<T>
where
    T: RouteState,
{
    /// Message for when the route is changed.
    BrowserNavigationRouteChanged((String, T)),
}

/// Input message type for interacting with the `RouteAgent'.
#[derive(Serialize, Deserialize, Debug)]
pub enum RouteRequest<T> {
    /// Replaces the most recent Route with a new one and alerts connected components to the route change.
    ReplaceRoute(RouteInfo<T>),
    /// Replaces the most recent Route with a new one, but does not alert connected components to the route change.
    ReplaceRouteNoBroadcast(RouteInfo<T>),
    /// Changes the route using a Route struct and alerts connected components to the route change.
    ChangeRoute(RouteInfo<T>),
    /// Changes the route using a Route struct, but does not alert connected components to the route change.
    ChangeRouteNoBroadcast(RouteInfo<T>),
    /// Gets the current route.
    GetCurrentRoute,
    /// Removes the entity from the Router Agent
    // TODO this is a temporary message because yew currently doesn't call the destructor, so it must be manually engaged
    Disconnect,
}

impl<T> Transferable for RouteRequest<T> where for<'de> T: Serialize + Deserialize<'de> {}



/// The Router agent holds on to the RouteService singleton and mediates access to it.
pub struct RouteAgent<T>
where
    for<'de> T: RouterState<'de>,
{
    link: AgentLink<RouteAgent<T>>,
    route_service: RouteService<T>,
    /// A list of all entities connected to the router.
    /// When a route changes, either initiated by the browser or by the app,
    /// the route change will be broadcast to all listening entities.
    subscribers: HashSet<HandlerId>,
}

impl <T: for<'de> RouterState<'de>>  Debug for RouteAgent<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("RouteAgent")
            .field("link", &"-")
            .field("route_service", &self.route_service)
            .field("subscribers", &self.subscribers.len())
            .finish()
    }
}


impl<T> Agent for RouteAgent<T>
where
    for<'de> T: RouterState<'de>,
{
    type Reach = Context;
    type Message = Msg<T>;
    type Input = RouteRequest<T>;
    type Output = RouteInfo<T>;

    fn create(link: AgentLink<Self>) -> Self {
        let callback = link.send_back(Msg::BrowserNavigationRouteChanged);
        let mut route_service = RouteService::new();
        route_service.register_callback(callback);

        RouteAgent {
            link,
            route_service,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::BrowserNavigationRouteChanged((_route_string, state)) => {
                trace!("Browser navigated");
                let mut route = RouteInfo::current_route(&self.route_service);
                route.state = Some(state);
                for sub in &self.subscribers {
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
            RouteRequest::ReplaceRoute(route) => {
                let route_string: String = route.route;
                self.route_service.replace_route(&route_string, route.state.unwrap_or_default());
                let route = RouteInfo::current_route(&self.route_service);
                for sub in &self.subscribers {
                    self.link.response(*sub, route.clone());
                }
            }
            RouteRequest::ReplaceRouteNoBroadcast(route) => {
                let route_string: String = route.route;
                self.route_service.replace_route(&route_string, route.state.unwrap_or_default());
            }
            RouteRequest::ChangeRoute(route) => {
                let route_string: String = route.route;
                // set the route
                self.route_service.set_route(&route_string, route.state.unwrap_or_default());
                // get the new route. This will contain a default state object
                let route = RouteInfo::current_route(&self.route_service);
                // broadcast it to all listening components
                for sub in &self.subscribers {
                    self.link.response(*sub, route.clone());
                }
            }
            RouteRequest::ChangeRouteNoBroadcast(route) => {
                let route_string: String = route.route;
                self.route_service.set_route(&route_string, route.state.unwrap_or_default());
            }
            RouteRequest::GetCurrentRoute => {
                let route = RouteInfo::current_route(&self.route_service);
                self.link.response(who, route.clone());
            }
            RouteRequest::Disconnect => {
                self.disconnected(who);
            }
        }
    }
    fn disconnected(&mut self, id: HandlerId) {
        trace!(
            "request to disconnect; num subs: {}",
            self.subscribers.len()
        );
        self.subscribers.remove(&id);
        trace!(
            "disconnect processed ; num subs: {}",
            self.subscribers.len()
        ); // the latter value should be -1
           // if it isn't then the handlerIds are different for each request
    }
}



/// A sender for the Router that doesn't send messages back to the component that connects to it.
///
/// This may be subject to change
pub struct RouteSenderAgent<T>
where
    for<'de> T: RouterState<'de>,
{
    router_agent: Box<dyn Bridge<RouteAgent<T>>>,
}



impl <T: for<'de> RouterState<'de>>  Debug for RouteSenderAgent<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("RouteSenderAgent")
            .field("router_agent", &"-")
//            .field("route_service", &self.route_service)
//            .field("subscribers", &self.subscribers.len())
            .finish()
    }
}

/// Non-instantiable type.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Void {}
impl Transferable for Void {}

impl<T> Agent for RouteSenderAgent<T>
where
    for<'de> T: RouterState<'de>,
{
    type Reach = Context;
    type Message = ();
    type Input = RouteRequest<T>;
    type Output = Void;

    fn create(link: AgentLink<Self>) -> Self {
        RouteSenderAgent {
            router_agent: RouteAgent::bridge(link.send_back(|_| ())),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle(&mut self, msg: Self::Input, _who: HandlerId) {
        self.router_agent.send(msg);
    }
}

/// Alias to RouteSenderBridge<()>;
pub type RouteSender = RouteSenderAgentBridge<()>;

/// A simplified interface to the router agent
pub struct RouteSenderAgentBridge<T>(Box<dyn Bridge<RouteSenderAgent<T>>>)
where
    for<'de> T: RouterState<'de>;


impl <T: for<'de> RouterState<'de>> Debug for RouteSenderAgentBridge<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_tuple("RouteSenderBridge")
            .finish()
    }
}

impl<T> RouteSenderAgentBridge<T>
where
    for<'de> T: RouterState<'de>,
{
    /// Creates a new sender only bridge.
    pub fn new(callback: Callback<Void>) -> Self {
        let router_agent = RouteSenderAgent::bridge(callback);
        RouteSenderAgentBridge(router_agent)
    }


    /// Sends a `RouteRequest` Message.
    pub fn send(&mut self, request: RouteRequest<T>) {
        self.0.send(request)
    }
}



/// A simplified interface to the router agent.
pub struct RouteAgentBridge<T>(Box<dyn Bridge<RouteAgent<T>>>)
    where
        for<'de> T: RouterState<'de>;


impl<T> RouteAgentBridge<T>
    where
            for<'de> T: RouterState<'de>,
{
    /// Creates a new bridge.
    pub fn new(callback: Callback<RouteInfo<T>>) -> Self {
        let router_agent = RouteAgent::bridge(callback);
        RouteAgentBridge(router_agent)
    }

    /// Experimental, may be removed
    ///
    /// Directly spawn a new Router
    pub fn spawn(callback: Callback<RouteInfo<T>>) -> Self {
        use yew::agent::Discoverer;
        let router_agent = Context::spawn_or_join(callback);
        RouteAgentBridge(router_agent)
    }

    /// Sends a `RouteRequest` Message.
    pub fn send(&mut self, request: RouteRequest<T>) {
        self.0.send(request)
    }
}

/// A wrapper around the bridge
//pub (crate) struct RouteAgentBridge<T: for<'de> YewRouterState<'de>>(pub Box<dyn Bridge<RouteAgent<T>>>);

impl <T: for<'de> RouterState<'de>> Debug for RouteAgentBridge<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_tuple("RouteAgentBridge")
            .finish()
    }
}

impl <T: for<'de> RouterState<'de>> Deref for RouteAgentBridge<T> {
    type Target = Box<dyn Bridge<RouteAgent<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl <T: for<'de> RouterState<'de>> DerefMut for RouteAgentBridge<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

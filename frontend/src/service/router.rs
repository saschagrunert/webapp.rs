//! Service to handle routing.

use failure::Error;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug, marker::PhantomData};
use stdweb::{
    unstable::TryFrom,
    web::{event::PopStateEvent, window, EventListenerHandle, History, IEventTarget, Location},
    JsSerialize, Value,
};
use yew::{callback::Callback, prelude::worker::*};

/// A service that facilitates manipulation of the browser's URL bar and
/// responding to browser 'forward' and 'back' events.
///
/// The `T` determines what route state can be stored in the route service.
pub struct RouterService<T> {
    history: History,
    location: Location,
    event_listener: Option<EventListenerHandle>,
    phantom_data: PhantomData<T>,
}

impl<T> RouterService<T>
where
    T: JsSerialize + Clone + TryFrom<Value> + 'static,
{
    /// Creates the route service.
    pub fn new() -> RouterService<T> {
        let location = window()
            .location()
            .expect("browser does not support location API");
        Self {
            history: window().history(),
            location,
            event_listener: None,
            phantom_data: PhantomData,
        }
    }

    /// Registers a callback to the route service. Callbacks will be called
    /// when the History API experiences a change such as popping a state off
    /// of its stack when the forward or back buttons are pressed.
    pub fn register_callback(&mut self, callback: Callback<(String, T)>) {
        self.event_listener = Some(window().add_event_listener(move |event: PopStateEvent| {
            let state_value: Value = event.state();

            if let Ok(state) = T::try_from(state_value) {
                if let Some(location) = window().location() {
                    let route: String = Self::get_route_from_location(&location);
                    callback.emit((route.clone(), state.clone()))
                }
            } else {
                eprintln!("Nothing farther back in history, not calling routing callback.");
            }
        }));
    }

    /// Sets the browser's url bar to contain the provided route, and creates a
    /// history entry that can be navigated via the forward and back buttons.
    /// The route should be a relative path that starts with a '/'. A state
    /// object be stored with the url.
    pub fn set_route(&mut self, route: &str, state: T) {
        self.history.push_state(state, "", Some(route));
    }

    fn get_route_from_location(location: &Location) -> String {
        let path = location.pathname().unwrap_or_else(|_| "".to_owned());
        let query = location.search().unwrap_or_else(|_| "".to_owned());
        let fragment = location.hash().unwrap_or_else(|_| "".to_owned());
        format!(
            "{path}{query}{fragment}",
            path = path,
            query = query,
            fragment = fragment
        )
    }

    /// Gets the concatenated path, query, and fragment strings
    pub fn get_route(&self) -> String {
        Self::get_route_from_location(&self.location)
    }

    /// Gets the path name of the current url.
    pub fn get_path(&self) -> Result<String, Error> {
        Ok(self.location.pathname()?)
    }

    /// Gets the query string of the current url.
    pub fn get_query(&self) -> Result<String, Error> {
        Ok(self.location.search()?)
    }

    /// Gets the fragment of the current url.
    pub fn get_fragment(&self) -> Result<String, Error> {
        Ok(self.location.hash()?)
    }
}

impl<T> Default for RouterService<T>
where
    T: JsSerialize + Clone + TryFrom<Value> + 'static,
{
    fn default() -> Self {
        RouterService::new()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Route<T> {
    pub path_segments: Vec<String>,
    pub query: Option<String>,
    pub fragment: Option<String>,
    pub state: T,
}

impl<T> Route<T>
where
    T: JsSerialize + Clone + TryFrom<Value> + Default + 'static,
{
    /// Convert to a String
    pub fn to_route_string(&self) -> String {
        let path = self.path_segments.join("/");
        let mut path = format!("/{}", path);
        if let Some(ref query) = self.query {
            path = format!("{}?{}", path, query);
        }
        if let Some(ref fragment) = self.fragment {
            path = format!("{}#{}", path, fragment)
        }
        path
    }

    /// Retrieve the current route
    pub fn current_route(route_service: &RouterService<T>) -> Result<Self, Error> {
        // guaranteed to always start with a '/'
        let path = route_service.get_path()?;
        let mut path_segments: Vec<String> = path.split('/').map(String::from).collect();
        // remove empty string that is split from the first '/'
        path_segments.remove(0);

        let mut query: String = route_service.get_query()?; // The first character will be a '?'
        let query: Option<String> = if query.len() > 1 {
            query.remove(0);
            Some(query)
        } else {
            None
        };

        let mut fragment: String = route_service.get_fragment()?; // The first character will be a '#'
        let fragment: Option<String> = if fragment.len() > 1 {
            fragment.remove(0);
            Some(fragment)
        } else {
            None
        };

        Ok(Route {
            path_segments,
            query,
            fragment,
            state: T::default(),
        })
    }
}

pub enum Message<T>
where
    T: JsSerialize + Clone + Debug + TryFrom<Value> + 'static,
{
    BrowserNavigationRouteChanged((String, T)),
}

impl<T> Transferable for Route<T> where for<'de> T: Serialize + Deserialize<'de> {}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request<T> {
    /// Changes the route using a RouteInfo struct and alerts connected
    /// components to the route change.
    ChangeRoute(Route<T>),

    /// Retrieve the current route request
    GetCurrentRoute,
}

impl<T> Transferable for Request<T> where for<'de> T: Serialize + Deserialize<'de> {}

/// The RouterAgent worker holds on to the RouterService singleton and mediates
/// access to it.
pub struct RouterAgent<T>
where
    for<'de> T: JsSerialize
        + Clone
        + Debug
        + TryFrom<Value>
        + Default
        + Serialize
        + Deserialize<'de>
        + 'static,
{
    link: AgentLink<RouterAgent<T>>,
    route_service: RouterService<T>,
    /// A list of all entities connected to the RouterAgent. When a route
    /// changes, either initiated by the browser or by the app, the route
    /// change will be broadcast to all listening entities.
    subscribers: HashSet<HandlerId>,
}

impl<T> Agent for RouterAgent<T>
where
    for<'de> T: JsSerialize
        + Clone
        + Debug
        + TryFrom<Value>
        + Default
        + Serialize
        + Deserialize<'de>
        + 'static,
{
    type Input = Request<T>;
    type Message = Message<T>;
    type Output = Route<T>;
    type Reach = Context;

    fn create(link: AgentLink<Self>) -> Self {
        let callback = link.send_back(Message::BrowserNavigationRouteChanged);
        let mut route_service = RouterService::default();
        route_service.register_callback(callback);

        Self {
            link,
            route_service,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::BrowserNavigationRouteChanged((_route_string, state)) => {
                if let Ok(mut route) = Route::current_route(&self.route_service) {
                    route.state = state;
                    for sub in &self.subscribers {
                        self.link.response(*sub, route.clone());
                    }
                }
            }
        }
    }

    fn handle(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            Request::ChangeRoute(route) => {
                let route_string: String = route.to_route_string();
                // set the route
                self.route_service.set_route(&route_string, route.state);
                // get the new route. This will contain a default state object
                if let Ok(route) = Route::current_route(&self.route_service) {
                    // broadcast it to all listening components
                    for sub in &self.subscribers {
                        self.link.response(*sub, route.clone());
                    }
                }
            }
            Request::GetCurrentRoute => {
                if let Ok(route) = Route::current_route(&self.route_service) {
                    self.link.response(who, route.clone());
                }
            }
        }
    }

    /// Add a client to the pool of connections of this agent
    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    /// Remove a client from the pool of connections of this agent
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

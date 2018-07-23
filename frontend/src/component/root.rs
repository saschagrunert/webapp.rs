//! The Root component as main entry point of the frontend application

use component::{content::ContentComponent, login::LoginComponent};
use route::RouterTarget;
use service::{
    api::{self, ApiAgent, ApiRequest, ApiResponse},
    cookie::CookieService,
    reducer::{ReducerAgent, ReducerRequest, ReducerResponse, ResponseType},
    router::{self, Route, RouterAgent},
    uikit::{NotificationStatus, UIkitService},
};
use string::{ERROR_SERVER_COMMUNICATION, ERROR_SERVER_INTERNAL, SERVER_COMMUNICATION_CLOSED};
use webapp::protocol::{request, response, Request, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Root Component
pub struct RootComponent {
    child_component: RouterTarget,
    api_agent: Box<Bridge<ApiAgent>>,
    reducer_agent: Box<Bridge<ReducerAgent>>,
    router_agent: Box<Bridge<RouterAgent<()>>>,
    cookie_service: CookieService,
    console_service: ConsoleService,
    uikit_service: UIkitService,
}

/// Available message types to process
pub enum Message {
    Api(ApiResponse),
    Route(Route<()>),
    Reducer(ReducerResponse),
}

impl Component for RootComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Create the reducer and subscribe to the used messages
        let mut reducer_agent = ReducerAgent::bridge(link.send_back(Message::Reducer));
        reducer_agent.send(ReducerRequest::Subscribe(vec![
            ResponseType::LoginSession,
            ResponseType::StatusClose,
            ResponseType::StatusError,
            ResponseType::StatusOpen,
        ]));

        let mut api_agent = ApiAgent::bridge(link.send_back(Message::Api));
        api_agent.send(ApiRequest::Subscribe(vec![
            api::ResponseType::Error,
            api::ResponseType::LoginCredentials,
        ]));

        // Return the component
        Self {
            child_component: RouterTarget::Loading,
            api_agent,
            reducer_agent,
            router_agent: RouterAgent::bridge(link.send_back(Message::Route)),
            console_service: ConsoleService::new(),
            cookie_service: CookieService::new(),
            uikit_service: UIkitService::new(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // Route to the appropriate child component
            Message::Route(route) => self.child_component = route.into(),
            // Handle API responses
            Message::Api(_) => {}
            // The WebSocket connection is open, try to authenticate if possible
            Message::Reducer(ReducerResponse::Open) => {
                // Test the API agent
                self.api_agent
                    .send(ApiRequest::Send(Request::Login(request::Login::Credentials {
                        username: "p".to_owned(),
                        password: "a".to_owned(),
                    })));

                // Verify if a session cookie already exist and try to authenticate if so
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    match Request::Login(request::Login::Session(Session { token })).to_vec() {
                        Some(data) => {
                            self.console_service.info("Token found, trying to authenticate");
                            self.reducer_agent.send(ReducerRequest::Send(data));
                        }
                        None => {
                            self.cookie_service.remove(SESSION_COOKIE);
                            self.router_agent
                                .send(router::Request::ChangeRoute(RouterTarget::Login.into()));
                        }
                    }
                } else {
                    self.console_service.info("No token found, routing to login");
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterTarget::Login.into()));
                }
            }
            // Received a response, handle if needed
            Message::Reducer(ReducerResponse::Data(response)) => match response {
                Response::Login(response::Login::Session(Ok(Session { token }))) => {
                    self.console_service.info("Session based login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    // Route to the content component
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterTarget::Content.into()));
                }
                Response::Login(response::Login::Session(Err(e))) => {
                    // Remote the existing cookie
                    self.console_service.info(&format!("Session based login failed: {}", e));
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterTarget::Login.into()));
                }
                Response::Error => {
                    // Send a notification to the user and route to the error page
                    self.uikit_service
                        .notify(ERROR_SERVER_INTERNAL, &NotificationStatus::Danger);
                }
                _ => {} // Not my response
            },
            // The root component also handles WebSocket failures like real errors
            Message::Reducer(ReducerResponse::Error) => {
                // Send a notification to the user
                self.uikit_service
                    .notify(ERROR_SERVER_COMMUNICATION, &NotificationStatus::Danger);

                // Route to the error child if coming from the loading child
                if self.child_component == RouterTarget::Loading {
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterTarget::Error.into()));
                }
            }
            // The root component also handles WebSocket failures like connection closings
            Message::Reducer(ReducerResponse::Close) => {
                // Send a notification to the user if app already in usage
                if self.child_component != RouterTarget::Error {
                    self.uikit_service
                        .notify(SERVER_COMMUNICATION_CLOSED, &NotificationStatus::Danger);
                }
            }
        }
        true
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        self.child_component.view()
    }
}

impl Renderable<RootComponent> for RouterTarget {
    fn view(&self) -> Html<RootComponent> {
        match *self {
            RouterTarget::Loading => html! {
                <div class="uk-position-center", uk-icon="icon: cloud-download; ratio: 3",></div>
            },
            RouterTarget::Login => html! {
               <LoginComponent:/>
            },
            RouterTarget::Content => html! {
               <ContentComponent:/>
            },
            RouterTarget::Error => html! {
                <div class="uk-position-center", uk-icon="icon: ban; ratio: 3",></div>
            },
        }
    }
}

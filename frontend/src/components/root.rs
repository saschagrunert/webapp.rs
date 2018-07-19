//! The Root component as main entry point of the frontend application

use components::{content::ContentComponent, login::LoginComponent, register::RegisterComponent};
use routes::RouterComponent;
use services::{
    cookie::CookieService,
    reducer::{ReducerAgent, ReducerRequest, ReducerResponse, ResponseType},
    router::{self, Route, RouterAgent},
    uikit::{NotificationStatus, UIkitService},
};
use webapp::protocol::{Login, Request, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Root Component
pub struct RootComponent {
    child_component: RouterComponent,
    reducer_agent: Box<Bridge<ReducerAgent>>,
    router_agent: Box<Bridge<RouterAgent<()>>>,
    cookie_service: CookieService,
    console_service: ConsoleService,
    uikit_service: UIkitService,
}

/// Available message types to process
pub enum Message {
    HandleRoute(Route<()>),
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

        // Return the component
        Self {
            child_component: RouterComponent::Loading,
            reducer_agent,
            router_agent: RouterAgent::bridge(link.send_back(Message::HandleRoute)),
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
            Message::HandleRoute(route) => self.child_component = route.into(),
            // The WebSocket connection is open, try to authenticate if possible
            Message::Reducer(ReducerResponse::Open) => {
                // Verify if a session cookie already exist and try to authenticate if so
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    match Request::Login(Login::Session(Session { token })).to_vec() {
                        Some(data) => {
                            self.console_service.info("Token found, trying to authenticate");
                            self.reducer_agent.send(ReducerRequest::Send(data));
                        }
                        None => {
                            self.cookie_service.remove(SESSION_COOKIE);
                            self.router_agent
                                .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                        }
                    }
                } else {
                    self.console_service.info("No token found, routing to login");
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                }
            }
            // Received a response, handle if needed
            Message::Reducer(ReducerResponse::Data(response)) => match response {
                Response::LoginSession(Ok(Session { token })) => {
                    self.console_service.info("Session based login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    // Route to the content component
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Content.into()));
                }
                Response::LoginSession(Err(e)) => {
                    // Remote the existing cookie
                    self.console_service.info(&format!("Session based login failed: {}", e));
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                }
                Response::Error => {
                    // Send a notification to the user and route to the error page
                    self.uikit_service
                        .notify("Internal server error", &NotificationStatus::Danger);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Error.into()));
                }
                _ => {} // Not my response
            },
            // The root component also handles WebSocket failures like real errors
            Message::Reducer(ReducerResponse::Error) => {
                // Send a notification to the user
                self.uikit_service
                    .notify("Server connection unavailable", &NotificationStatus::Danger);

                // Route to the error child if coming from the loading child
                if self.child_component == RouterComponent::Loading {
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Error.into()));
                }
            }
            // The root component also handles WebSocket failures like connection closings
            Message::Reducer(ReducerResponse::Close) => {
                // Send a notification to the user if app already in usage
                if self.child_component != RouterComponent::Error {
                    self.uikit_service
                        .notify("Server connection closed", &NotificationStatus::Danger);
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

impl Renderable<RootComponent> for RouterComponent {
    fn view(&self) -> Html<RootComponent> {
        match *self {
            RouterComponent::Loading => html! {
                <div class="uk-position-center", uk-icon="icon: cloud-download; ratio: 3",></div>
            },
            RouterComponent::Login => html! {
               <LoginComponent:/>
            },
            RouterComponent::Register => html! {
               <RegisterComponent:/>
            },
            RouterComponent::Content => html! {
               <ContentComponent:/>
            },
            RouterComponent::Error => html! {
                <div class="uk-position-center", uk-icon="icon: ban; ratio: 3",></div>
            },
        }
    }
}

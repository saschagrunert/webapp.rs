//! The Main Content component

use route::RouterTarget;
use service::{
    cookie::CookieService,
    reducer::{ReducerAgent, ReducerRequest, ReducerResponse, ResponseType},
    router::{self, RouterAgent},
};
use string::{TEXT_CONTENT, TEXT_LOGOUT};
use webapp::protocol::{Request, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Content component
pub struct ContentComponent {
    logout_button_disabled: bool,
    reducer_agent: Box<Bridge<ReducerAgent>>,
    router_agent: Box<Bridge<RouterAgent<()>>>,
    console_service: ConsoleService,
    cookie_service: CookieService,
}

/// Available message types to process
pub enum Message {
    Ignore,
    LogoutRequest,
    Reducer(ReducerResponse),
}

impl Component for ContentComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Guard the authentication
        let mut router_agent = RouterAgent::bridge(link.send_back(|_| Message::Ignore));
        let cookie_service = CookieService::new();
        let mut console_service = ConsoleService::new();
        if cookie_service.get(SESSION_COOKIE).is_err() {
            console_service.log("No session token found, routing back to login");
            router_agent.send(router::Request::ChangeRoute(RouterTarget::Login.into()));
        }

        // Create the reducer and subscribe to the used messages
        let mut reducer_agent = ReducerAgent::bridge(link.send_back(Message::Reducer));
        reducer_agent.send(ReducerRequest::Subscribe(vec![
            ResponseType::Logout,
            ResponseType::StatusClose,
            ResponseType::StatusError,
        ]));

        // Return the component
        Self {
            logout_button_disabled: false,
            reducer_agent,
            router_agent,
            console_service,
            cookie_service,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogoutRequest => if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                // Create the logout request
                match Request::Logout(Session { token }).to_vec() {
                    Some(data) => {
                        // Disable user interaction
                        self.logout_button_disabled = true;

                        // Send the request
                        self.reducer_agent.send(ReducerRequest::Send(data));
                    }
                    None => self.console_service.error("Unable to write logout request"),
                }
            } else {
                // It should not happen but in case there is no session cookie on logout, route
                // back to login
                self.console_service.error("No session cookie found");
                self.router_agent
                    .send(router::Request::ChangeRoute(RouterTarget::Login.into()));
            },
            Message::Reducer(ReducerResponse::Data(response)) => match response {
                Response::Logout(Ok(())) => {
                    self.console_service.log("Got valid logout response");
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterTarget::Login.into()));
                }
                Response::Logout(Err(e)) => self.console_service.info(&format!("Unable to logout: {}", e)),
                _ => {} // Not my response
            },
            Message::Reducer(ReducerResponse::Close) | Message::Reducer(ReducerResponse::Error) => {
                self.logout_button_disabled = true
            }
            _ => {}
        }
        true
    }
}

impl Renderable<ContentComponent> for ContentComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <h1 class="uk-card-title",>{TEXT_CONTENT}</h1>
                <button disabled=self.logout_button_disabled,
                    class="uk-button uk-button-default",
                    onclick=|_| Message::LogoutRequest,>{TEXT_LOGOUT}</button>
            </div>
        }
    }
}

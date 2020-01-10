//! The Main Content component

use crate::{
    api::Response,
    route::RouterTarget,
    service::{
        cookie::CookieService,
        session_timer::{self, SessionTimerAgent},
        uikit::{NotificationStatus, UIkitService},
    },
    string::{REQUEST_ERROR, RESPONSE_ERROR, TEXT_CONTENT, TEXT_LOGOUT},
    SESSION_COOKIE,
};
use log::{error, info, warn};
use webapp::{
    protocol::{model::Session, request, response},
    API_URL_LOGOUT,
};
use yew::{agent::Bridged, format::Json, html, prelude::*, services::fetch::FetchTask};
use yew_router::agent::{RouteAgent, RouteRequest::ChangeRoute};

/// Data Model for the Content component
pub struct ContentComponent {
    component_link: ComponentLink<ContentComponent>,
    cookie_service: CookieService,
    fetch_task: Option<FetchTask>,
    logout_button_disabled: bool,
    router_agent: Box<dyn Bridge<RouteAgent<()>>>,
    session_timer_agent: Box<dyn Bridge<SessionTimerAgent>>,
    uikit_service: UIkitService,
}

/// Available message types to process
pub enum Message {
    Fetch(Response<response::Logout>),
    Ignore,
    LogoutRequest,
}

impl Component for ContentComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Guard the authentication
        let mut router_agent = RouteAgent::bridge(link.callback(|_| Message::Ignore));
        let cookie_service = CookieService::new();
        let mut session_timer_agent = SessionTimerAgent::bridge(link.callback(|_| Message::Ignore));
        if cookie_service.get(SESSION_COOKIE).is_err() {
            info!("No session token found, routing back to login");
            router_agent.send(ChangeRoute(RouterTarget::Login.into()));
        } else {
            // Start the timer to keep the session active
            session_timer_agent.send(session_timer::Request::Start);
        }

        // Return the component
        Self {
            component_link: link,
            cookie_service,
            fetch_task: None,
            logout_button_disabled: false,
            router_agent,
            session_timer_agent,
            uikit_service: UIkitService::new(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogoutRequest => {
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    self.fetch_task = fetch! {
                        request::Logout(Session::new(token)) => API_URL_LOGOUT,
                        self.component_link, Message::Fetch,
                        || {
                            // Disable user interaction
                            self.logout_button_disabled = true;
                        },
                        || {
                            error!("Unable to create logout request");
                            self.uikit_service
                                .notify(REQUEST_ERROR, &NotificationStatus::Danger);
                        }
                    };
                } else {
                    // It should not happen but in case there is no session cookie on logout, route
                    // back to login
                    error!("No session cookie found");
                    self.router_agent
                        .send(ChangeRoute(RouterTarget::Login.into()));
                }
            }

            // The message for all fetch responses
            Message::Fetch(response) => {
                let (meta, Json(body)) = response.into_parts();

                // Check the response type
                if meta.status.is_success() {
                    match body {
                        Ok(response::Logout) => info!("Got valid logout response"),
                        _ => {
                            warn!("Got wrong logout response");
                            self.uikit_service
                                .notify(RESPONSE_ERROR, &NotificationStatus::Danger);
                        }
                    }
                } else {
                    warn!("Logout failed with status: {}", meta.status);
                }

                // Remove the existing cookie
                self.cookie_service.remove(SESSION_COOKIE);
                self.session_timer_agent.send(session_timer::Request::Stop);
                self.router_agent
                    .send(ChangeRoute(RouterTarget::Login.into()));
                self.logout_button_disabled = true;

                // Remove the ongoing task
                self.fetch_task = None;
            }
            Message::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        let onclick = self.component_link.callback(|_| Message::LogoutRequest);
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <h1 class="uk-card-title",>{TEXT_CONTENT}</h1>
                <button disabled=self.logout_button_disabled,
                    class="uk-button uk-button-default",
                    onclick=onclick>{TEXT_LOGOUT}</button>
            </div>
        }
    }
}

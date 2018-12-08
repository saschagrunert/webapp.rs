//! The Root component as main entry point of the frontend application

use crate::{
    api::Response,
    component::{content::ContentComponent, login::LoginComponent},
    route::RouterTarget,
    service::{
        cookie::CookieService,
        uikit::{NotificationStatus, UIkitService},
    },
    string::{REQUEST_ERROR, RESPONSE_ERROR},
    SESSION_COOKIE,
};
use webapp::{
    protocol::{model::Session, request::LoginSession, response::Login},
    API_URL_LOGIN_SESSION,
};
use yew::{format::Cbor, prelude::*, services::fetch::FetchTask};
use yew_router::{self, Route, RouterAgent};

/// Data Model for the Root Component
pub struct RootComponent {
    child_component: RouterTarget,
    cookie_service: CookieService,
    fetch_task: Option<FetchTask>,
    router_agent: Box<Bridge<RouterAgent<()>>>,
    uikit_service: UIkitService,
}

/// Available message types to process
pub enum Message {
    Fetch(Response<Login>),
    Route(Route<()>),
}

impl Component for RootComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        // Create needed services
        let cookie_service = CookieService::new();
        let mut fetch_task = None;
        let mut router_agent = RouterAgent::bridge(link.send_back(Message::Route));
        let uikit_service = UIkitService::new();

        // Verify if a session cookie already exist and try to authenticate if so
        if let Ok(token) = cookie_service.get(SESSION_COOKIE) {
            fetch_task = fetch! {
                LoginSession(Session::new(token)) => API_URL_LOGIN_SESSION,
                link, Message::Fetch,
                || {},
                || {
                    error!("Unable to create session login request");
                    uikit_service.notify(REQUEST_ERROR, &NotificationStatus::Danger);
                    cookie_service.remove(SESSION_COOKIE);
                    router_agent.send(yew_router::Request::ChangeRoute(RouterTarget::Login.into()));
                }
            };
        } else {
            info!("No token found, routing to login");
            router_agent.send(yew_router::Request::ChangeRoute(RouterTarget::Login.into()));
        }

        // Return the component
        Self {
            child_component: RouterTarget::Loading,
            cookie_service,
            fetch_task,
            router_agent,
            uikit_service,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // Route to the appropriate child component
            Message::Route(route) => self.child_component = route.into(),

            // The message for all fetch responses
            Message::Fetch(response) => {
                let (meta, Cbor(body)) = response.into_parts();

                // Check the response type
                if meta.status.is_success() {
                    match body {
                        Ok(Login(Session { token })) => {
                            info!("Session based login succeed");

                            // Set the retrieved session cookie
                            self.cookie_service.set(SESSION_COOKIE, &token);

                            // Route to the content component
                            self.router_agent.send(yew_router::Request::ChangeRoute(
                                RouterTarget::Content.into(),
                            ));
                        }
                        _ => {
                            // Send an error notification to the user on any failure
                            warn!("Got wrong session login response");
                            self.uikit_service
                                .notify(RESPONSE_ERROR, &NotificationStatus::Danger);
                            self.router_agent
                                .send(yew_router::Request::ChangeRoute(RouterTarget::Login.into()));
                        }
                    }
                } else {
                    // Remove the existing cookie
                    warn!("Session login failed with status: {}", meta.status);
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(yew_router::Request::ChangeRoute(RouterTarget::Login.into()));
                }

                // Remove the ongoing task
                self.fetch_task = None;
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
            RouterTarget::Loading => {
                html! {
                    <div class="uk-position-center", uk-icon="icon: cloud-download; ratio: 3",></div>
                }
            }
            RouterTarget::Login => {
                html! {
                    <LoginComponent:/>
                }
            }
            RouterTarget::Content => {
                html! {
                    <ContentComponent:/>
                }
            }
            RouterTarget::Error => {
                html! {
                    <div class="uk-position-center", uk-icon="icon: ban; ratio: 3",></div>
                }
            }
        }
    }
}

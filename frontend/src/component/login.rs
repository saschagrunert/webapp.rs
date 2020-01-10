//! The Login component

use crate::{
    api::Response,
    route::RouterTarget,
    service::{
        cookie::CookieService,
        uikit::{NotificationStatus, UIkitService},
    },
    string::{
        AUTHENTICATION_ERROR, INPUT_PASSWORD, INPUT_USERNAME, REQUEST_ERROR, RESPONSE_ERROR,
        TEXT_LOGIN,
    },
    SESSION_COOKIE,
};
use log::{error, info, warn};
use webapp::{
    protocol::{model::Session, request::LoginCredentials, response::Login},
    API_URL_LOGIN_CREDENTIALS,
};
use yew::{format::Json, html, prelude::*, services::fetch::FetchTask};
use yew_router::agent::{RouteAgent, RouteRequest::ChangeRoute};

/// Data Model for the Login component
pub struct LoginComponent {
    component_link: ComponentLink<LoginComponent>,
    cookie_service: CookieService,
    fetch_task: Option<FetchTask>,
    inputs_disabled: bool,
    login_button_disabled: bool,
    password: String,
    router_agent: Box<dyn Bridge<RouteAgent<()>>>,
    uikit_service: UIkitService,
    username: String,
}

/// Available message types to process
pub enum Message {
    Fetch(Response<Login>),
    Ignore,
    LoginRequest,
    UpdatePassword(String),
    UpdateUsername(String),
}

impl Component for LoginComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Return the component
        Self {
            cookie_service: CookieService::new(),
            fetch_task: None,
            inputs_disabled: false,
            login_button_disabled: true,
            password: String::new(),
            router_agent: RouteAgent::bridge(link.callback(|_| Message::Ignore)),
            component_link: link,
            uikit_service: UIkitService::new(),
            username: String::new(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // Login via username and password
            Message::LoginRequest => {
                self.fetch_task = fetch! {
                    LoginCredentials {
                        username: self.username.to_owned(),
                        password: self.password.to_owned(),
                    } => API_URL_LOGIN_CREDENTIALS,
                    self.component_link, Message::Fetch,
                    || {
                        // Disable user interaction
                        self.login_button_disabled = true;
                        self.inputs_disabled = true;
                    },
                    || {
                        error!("Unable to create credentials login request");
                        self.uikit_service
                            .notify(REQUEST_ERROR, &NotificationStatus::Danger);
                    }
                };
            }

            Message::UpdateUsername(new_username) => {
                self.username = new_username;
                self.update_button_state();
            }

            Message::UpdatePassword(new_password) => {
                self.password = new_password;
                self.update_button_state();
            }

            // The message for all fetch responses
            Message::Fetch(response) => {
                let (meta, Json(body)) = response.into_parts();

                // Check the response type
                if meta.status.is_success() {
                    match body {
                        Ok(Login(Session { token })) => {
                            info!("Credential based login succeed");

                            // Set the retrieved session cookie
                            self.cookie_service.set(SESSION_COOKIE, &token);

                            // Route to the content component
                            self.router_agent
                                .send(ChangeRoute(RouterTarget::Content.into()));
                        }
                        _ => {
                            warn!("Got wrong credentials login response");
                            self.uikit_service
                                .notify(RESPONSE_ERROR, &NotificationStatus::Danger);
                        }
                    }
                } else {
                    // Authentication failed
                    warn!("Credentials login failed with status: {}", meta.status);
                    self.uikit_service
                        .notify(AUTHENTICATION_ERROR, &NotificationStatus::Warning);
                    self.login_button_disabled = false;
                    self.inputs_disabled = false;
                }

                // Remove the ongoing task
                self.fetch_task = None;
            }
            Message::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        let onclick = self.component_link.callback(|_| Message::LoginRequest);
        let oninput_username = self
            .component_link
            .callback(|e: InputData| Message::UpdateUsername(e.value));
        let oninput_password = self
            .component_link
            .callback(|e: InputData| Message::UpdatePassword(e.value));
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <h1 class="uk-card-title",>{TEXT_LOGIN}</h1>
                <form>
                    <fieldset class="uk-fieldset",>
                        <input class="uk-input uk-margin",
                            placeholder=INPUT_USERNAME,
                            disabled=self.inputs_disabled,
                            value=&self.username,
                            oninput=oninput_username />
                        <input class="uk-input uk-margin-bottom",
                            type="password",
                            placeholder=INPUT_PASSWORD,
                            disabled=self.inputs_disabled,
                            value=&self.password,
                            oninput=oninput_password />
                        <button class="uk-button uk-button-primary",
                            type="submit",
                            disabled=self.login_button_disabled,
                            onclick=onclick>{TEXT_LOGIN}</button>
                    </fieldset>
                </form>
            </div>
        }
    }
}

impl LoginComponent {
    fn update_button_state(&mut self) {
        self.login_button_disabled = self.username.is_empty() || self.password.is_empty();
    }
}

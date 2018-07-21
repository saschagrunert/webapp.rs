//! The Login component

use route::RouterTarget;
use service::{
    cookie::CookieService,
    reducer::{ReducerAgent, ReducerRequest, ReducerResponse, ResponseType},
    router::{self, RouterAgent},
    uikit::{NotificationStatus, UIkitService},
};
use string::{ERROR_AUTHENTICATION_FAILED, INPUT_PASSWORD, INPUT_USERNAME, TEXT_LOGIN, TEXT_REGISTER};
use webapp::protocol::{request, response, Request, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    login_button_disabled: bool,
    inputs_and_register_button_disabled: bool,
    reducer_agent: Box<Bridge<ReducerAgent>>,
    router_agent: Box<Bridge<RouterAgent<()>>>,
    console_service: ConsoleService,
    cookie_service: CookieService,
    uikit_service: UIkitService,
}

/// Available message types to process
pub enum Message {
    Ignore,
    LoginRequest,
    RegisterRequest,
    UpdatePassword(String),
    UpdateUsername(String),
    Reducer(ReducerResponse),
}

impl Component for LoginComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Create the reducer and subscribe to the used messages
        let mut reducer_agent = ReducerAgent::bridge(link.send_back(Message::Reducer));
        reducer_agent.send(ReducerRequest::Subscribe(vec![
            ResponseType::LoginCredentials,
            ResponseType::StatusClose,
            ResponseType::StatusError,
        ]));

        // Return the component
        Self {
            username: String::new(),
            password: String::new(),
            login_button_disabled: true,
            inputs_and_register_button_disabled: false,
            reducer_agent,
            router_agent: RouterAgent::bridge(link.send_back(|_| Message::Ignore)),
            console_service: ConsoleService::new(),
            cookie_service: CookieService::new(),
            uikit_service: UIkitService::new(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // Login via username and password
            Message::LoginRequest => match Request::Login(request::Login::Credentials {
                username: self.username.to_owned(),
                password: self.password.to_owned(),
            }).to_vec()
            {
                Some(data) => {
                    // Disable user interaction
                    self.login_button_disabled = true;
                    self.inputs_and_register_button_disabled = true;

                    // Send the request
                    self.reducer_agent.send(ReducerRequest::Send(data));
                }
                None => {
                    self.console_service.error("Unable to create login credential request");
                }
            },
            // Route to the register component
            Message::RegisterRequest => self
                .router_agent
                .send(router::Request::ChangeRoute(RouterTarget::Register.into())),
            Message::UpdateUsername(new_username) => {
                self.username = new_username;
                self.update_button_state();
            }
            Message::UpdatePassword(new_password) => {
                self.password = new_password;
                self.update_button_state();
            }
            Message::Reducer(ReducerResponse::Data(response)) => match response {
                Response::Login(response::Login::Credentials(Ok(Session { token }))) => {
                    self.console_service.info("Credential based login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    // Route to the content component
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterTarget::Content.into()));
                }
                Response::Login(response::Login::Credentials(Err(e))) => {
                    self.console_service
                        .warn(&format!("Credential based login failed: {}", e));
                    self.uikit_service
                        .notify(ERROR_AUTHENTICATION_FAILED, &NotificationStatus::Warning);
                    self.login_button_disabled = false;
                    self.inputs_and_register_button_disabled = false;
                }
                _ => {} // Not my response
            },
            Message::Reducer(ReducerResponse::Close) | Message::Reducer(ReducerResponse::Error) => {
                self.login_button_disabled = true;
                self.inputs_and_register_button_disabled = true;
            }
            _ => {}
        }
        true
    }
}

impl LoginComponent {
    fn update_button_state(&mut self) {
        self.login_button_disabled = self.username.is_empty() || self.password.is_empty();
    }
}

impl Renderable<LoginComponent> for LoginComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <h1 class="uk-card-title",>{TEXT_LOGIN}</h1>
                <form onsubmit="return false",>
                    <fieldset class="uk-fieldset",>
                        <input class="uk-input uk-margin",
                            placeholder=INPUT_USERNAME,
                            disabled=self.inputs_and_register_button_disabled,
                            value=&self.username,
                            oninput=|e| Message::UpdateUsername(e.value), />
                        <input class="uk-input uk-margin-bottom",
                            type="password",
                            placeholder=INPUT_PASSWORD,
                            disabled=self.inputs_and_register_button_disabled,
                            value=&self.password,
                            oninput=|e| Message::UpdatePassword(e.value), />
                        <div class="uk-button-group",>
                            <button class="uk-button uk-button-primary",
                                type="submit",
                                disabled=self.login_button_disabled,
                                onclick=|_| Message::LoginRequest,>{TEXT_LOGIN}</button>
                            <button class="uk-button uk-button-default",
                                type="register",
                                disabled=self.inputs_and_register_button_disabled,
                                onclick=|_| Message::RegisterRequest,>{TEXT_REGISTER}</button>
                        </div>
                    </fieldset>
                </form>
            </div>
        }
    }
}

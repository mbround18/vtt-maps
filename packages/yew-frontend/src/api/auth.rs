use gloo_net::http::Request;
use serde::Deserialize;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::Reducible;
use yew::prelude::*;

const AUTH_ME_ENDPOINT: &str = "/api/auth/me";
const AUTH_LOGOUT_ENDPOINT: &str = "/api/auth/logout";
const AUTH_START_ENDPOINT: &str = "/api/auth/discord/start";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Contributor,
    Guest,
}

impl UserRole {
    pub fn label(&self) -> &'static str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::Contributor => "Contributor",
            UserRole::Guest => "Guest",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Contributor => "contributor",
            UserRole::Guest => "guest",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
}

impl UserProfile {
    pub fn fallback_initial(&self) -> String {
        self.username
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "?".into())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AuthState {
    pub user: Option<UserProfile>,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            user: None,
            loading: true,
            error: None,
        }
    }
}

pub enum AuthAction {
    StartLoading,
    Loaded(Option<UserProfile>),
    Failed(String),
    LoggedOut,
}

impl Reducible for AuthState {
    type Action = AuthAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let state = (*self).clone();
        Rc::new(match action {
            AuthAction::StartLoading => AuthState {
                loading: true,
                error: None,
                ..state
            },
            AuthAction::Loaded(user) => AuthState {
                user,
                loading: false,
                error: None,
            },
            AuthAction::Failed(err) => AuthState {
                loading: false,
                error: Some(err),
                ..state
            },
            AuthAction::LoggedOut => AuthState {
                user: None,
                loading: false,
                error: None,
            },
        })
    }
}

#[derive(Clone)]
pub struct AuthContextValue {
    pub state: AuthState,
    pub login: Callback<()>,
    pub logout: Callback<()>,
    #[allow(dead_code)]
    pub refresh: Callback<()>,
}

impl PartialEq for AuthContextValue {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let state = use_reducer_eq(AuthState::default);

    let refresh = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(AuthAction::StartLoading);
            let state = state.clone();
            spawn_local(async move {
                match fetch_current_user().await {
                    Ok(user) => state.dispatch(AuthAction::Loaded(user)),
                    Err(err) => state.dispatch(AuthAction::Failed(err)),
                }
            });
        })
    };

    {
        let refresh_on_mount = refresh.clone();
        use_effect_with((), move |_| {
            refresh_on_mount.emit(());
            || {}
        });
    }

    let logout = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            spawn_local(async move {
                match perform_logout().await {
                    Ok(()) => state.dispatch(AuthAction::LoggedOut),
                    Err(err) => state.dispatch(AuthAction::Failed(err)),
                }
            });
        })
    };

    let login = Callback::from(move |_| {
        if let Some(win) = window() {
            let _ = win.location().set_href(AUTH_START_ENDPOINT);
        }
    });

    let context = AuthContextValue {
        state: (*state).clone(),
        login,
        logout,
        refresh,
    };

    html! {
        <ContextProvider<AuthContextValue> context={context}>
            { for props.children.iter() }
        </ContextProvider<AuthContextValue>>
    }
}

#[hook]
pub fn use_auth() -> AuthContextValue {
    use_context::<AuthContextValue>().expect("AuthProvider is missing from the component tree")
}

async fn fetch_current_user() -> Result<Option<UserProfile>, String> {
    let response = Request::get(AUTH_ME_ENDPOINT)
        .send()
        .await
        .map_err(|e| format!("failed to call /auth/me: {e}"))?;

    match response.status() {
        200 => response
            .json::<UserProfile>()
            .await
            .map(Some)
            .map_err(|e| format!("invalid auth payload: {e}")),
        401 => Ok(None),
        status => {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("session fetch failed ({status}): {message}"))
        }
    }
}

async fn perform_logout() -> Result<(), String> {
    let response = Request::post(AUTH_LOGOUT_ENDPOINT)
        .send()
        .await
        .map_err(|e| format!("failed to call /auth/logout: {e}"))?;

    match response.status() {
        200..=299 => Ok(()),
        status => Err(format!("logout failed with status {status}")),
    }
}

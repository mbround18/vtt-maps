use crate::api::auth::use_auth;
use web_sys::MouseEvent;
use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    let is_open = use_state(|| false);
    let stars = use_state(|| None::<u32>);
    let auth = use_auth();

    // Fetch GitHub stars on component mount
    {
        let stars = stars.clone();
        use_effect(move || {
            let stars_state = stars.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) =
                    gloo_net::http::Request::get("https://api.github.com/repos/dnd-apps/vtt-maps")
                        .send()
                        .await
                    && let Ok(json) = response.json::<serde_json::Value>().await
                    && let Some(stargazers_count) = json
                        .get("stargazers_count")
                        .and_then(serde_json::Value::as_u64)
                {
                    #[allow(clippy::cast_possible_truncation)]
                    stars_state.set(Some(stargazers_count as u32));
                }
            });
            || {}
        });
    }

    let on_toggle = {
        let is_open = is_open.clone();
        Callback::from(move |_: MouseEvent| {
            is_open.set(!*is_open);
        })
    };

    let close_nav = {
        let is_open = is_open.clone();
        Callback::from(move |_| {
            is_open.set(false);
        })
    };

    let close_nav_event = {
        let close_nav = close_nav.clone();
        Callback::from(move |_: MouseEvent| {
            close_nav.emit(());
        })
    };

    let nav_class = if *is_open {
        "navbar-menu active"
    } else {
        "navbar-menu"
    };

    let login_action = {
        let login = auth.login.clone();
        let close_nav = close_nav.clone();
        Callback::from(move |_: MouseEvent| {
            login.emit(());
            close_nav.emit(());
        })
    };

    let logout_action = {
        let logout = auth.logout.clone();
        let close_nav = close_nav.clone();
        Callback::from(move |_: MouseEvent| {
            logout.emit(());
            close_nav.emit(());
        })
    };

    let profile_node = match (auth.state.loading, auth.state.user.clone()) {
        (true, _) => html! {
            <li class="nav-item nav-auth">
                <span class="nav-link auth-status">{"Syncing session..."}</span>
            </li>
        },
        (_, Some(user)) => {
            let logout_click = logout_action.clone();
            html! {
                <li class="nav-item nav-auth">
                    <div class="profile-chip">
                        {
                            if let Some(avatar) = user.avatar_url.clone() {
                                html! { <img class="profile-avatar" src={avatar} alt="Discord avatar" /> }
                            } else {
                                html! { <div class="profile-avatar fallback">{user.fallback_initial()}</div> }
                            }
                        }
                        <div class="profile-meta">
                            <span class="profile-name">{user.username.clone()}</span>
                            <span class={classes!("role-badge", user.role.css_class())}>{user.role.label()}</span>
                        </div>
                        <button type="button" class="logout-button" onclick={logout_click}>{"Logout"}</button>
                    </div>
                </li>
            }
        }
        (_, None) => html! {
            <li class="nav-item nav-auth">
                <button type="button" class="nav-link nav-cta login-button" onclick={login_action.clone()}>
                    {"Login with Discord"}
                </button>
            </li>
        },
    };

    let error_node: Html = auth
        .state
        .error
        .clone()
        .map(|err| {
            html! {
                <li class="nav-item nav-auth auth-error">
                    <span>{err}</span>
                </li>
            }
        })
        .unwrap_or_default();

    html! {
        <header class="site-header">
          <div class="container">
            <div class="header-inner">
              <a href="/" class="logo" onclick={close_nav_event.clone()}>
                <img src="/assets/vtt-maps-logo.png" alt="VTT Maps Logo"/>
                <span class="site-title">{"VTT Maps"}</span>
              </a>
              <button class="navbar-toggle" onclick={on_toggle.clone()}>
                <span class="sr-only">{"Toggle navigation"}</span>
                <div class={ if *is_open { "hamburger active" } else { "hamburger" } }>
                  <span class="bar"></span><span class="bar"></span><span class="bar"></span>
                </div>
              </button>
              <nav class={nav_class}>
                <ul class="nav-list">
                  <li class="nav-item"><a href="/" class="nav-link" onclick={close_nav_event.clone()}>{"Home 🏠"}</a></li>
                  <li class="nav-item"><a href="/catalog" class="nav-link" onclick={close_nav_event.clone()}>{"Catalog 📖"}</a></li>
                  <li class="nav-item">
                    <a href="https://github.com/dnd-apps/vtt-maps" target="_blank" class="nav-link">
                      {"GitHub"}
                      {
                        if let Some(count) = *stars {
                            html! { <span class="github-stars">{format!(" ⭐ {} ⭐", count)}</span> }
                        } else {
                            html! {}
                        }
                      }
                    </a>
                  </li>
                  <li class="nav-item">
                    <a href="https://github.com/sponsors/mbround18" target="_blank" class="nav-link nav-cta">
                      {"Support ❤️"}
                    </a>
                  </li>
                  {profile_node}
                  {error_node}
                </ul>
              </nav>
            </div>
          </div>
          <div class="wave-divider"></div>
        </header>
    }
}

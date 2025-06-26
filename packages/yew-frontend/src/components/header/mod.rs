use yew::prelude::*;
use yew_hooks::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    let is_open = use_bool_toggle(false);
    let nav_ref = use_node_ref();
    let stars = use_state(|| None::<u32>);

    {
        let stars = stars.clone();
        use_effect_once(move || {
            let stars = stars.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) =
                    gloo_net::http::Request::get("https://api.github.com/repos/dnd-apps/vtt-maps")
                        .send()
                        .await
                {
                    if let Ok(json) = response.json::<serde_json::Value>().await {
                        if let Some(stargazers_count) = json
                            .get("stargazers_count")
                            .and_then(serde_json::Value::as_u64)
                        {
                            #[allow(clippy::cast_possible_truncation)]
                            stars.set(Some(stargazers_count as u32));
                        }
                    }
                }
            });
            || {}
        });
    }

    let on_toggle = {
        let is_open = is_open.clone();
        Callback::from(move |_| {
            is_open.toggle();
        })
    };

    let on_close = {
        let is_open = is_open.clone();
        Callback::from(move |_| {
            if *is_open {
                is_open.toggle();
            }
        })
    };

    {
        let is_open = is_open.clone();
        use_click_away(nav_ref.clone(), move |_| {
            if *is_open {
                is_open.toggle();
            }
        });
    }
    {
        let is_open = is_open.clone();
        use_event_with_window("resize", move |_: web_sys::Event| {
            if *is_open {
                is_open.toggle();
            }
        });
    }

    let nav_class = if *is_open {
        "navbar-menu active"
    } else {
        "navbar-menu"
    };

    html! {
        <header class="site-header">
          <div class="container">
            <div class="header-inner">
              <a href="/" class="logo" onclick={on_close.clone()}>
                <img src="/assets/vtt-maps-logo.png" alt="VTT Maps Logo"/>
                <span class="site-title">{"VTT Maps"}</span>
              </a>
              <button class="navbar-toggle" onclick={on_toggle.clone()}>
                <span class="sr-only">{"Toggle navigation"}</span>
                <div class={ if *is_open { "hamburger active" } else { "hamburger" } }>
                  <span class="bar"></span><span class="bar"></span><span class="bar"></span>
                </div>
              </button>
              <nav class={nav_class} ref={nav_ref.clone()}>
                <ul class="nav-list">
                  <li class="nav-item"><a href="/" class="nav-link" onclick={on_close.clone()}>{"Home 🏠"}</a></li>
                  <li class="nav-item"><a href="/catalog" class="nav-link" onclick={on_close.clone()}>{"Catalog 📖"}</a></li>
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
                </ul>
              </nav>
            </div>
          </div>
          <div class="wave-divider"></div>
        </header>
    }
}

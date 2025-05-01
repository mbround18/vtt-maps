use crate::utils::api;
use crate::utils::externals::updateReadmeAnchors;
use gloo_console::log;
use yew::prelude::*;

#[function_component(ReadMe)]
pub fn readme() -> Html {
    let content = use_state(|| String::new());

    {
        let content = content.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::get("/docs/readme").send().await {
                    Ok(response) if response.ok() => match response.text().await {
                        Ok(txt) => {
                            content.set(txt);
                            updateReadmeAnchors();
                        }
                        Err(e) => log!("Failed to read body:", e.to_string()),
                    },
                    Ok(response) => log!("Fetch failed, status:", response.status()),
                    Err(e) => log!("Network error:", e.to_string()),
                }
            });
            || ()
        });
    }

    let inner = Html::from_html_unchecked(AttrValue::from((content.clone()).to_string()));

    html! {
        <div id="readme" class="card" tabindex="0">
            if content.is_empty() {
                <div class="loading">{"Loading README content..."}</div>
            } else {
                <div>{inner}</div>
            }
        </div>
    }
}

// src/components/markdown_viewer.rs
use crate::api::context::ApiEndpoint;
use crate::utils::externals::updateReadmeAnchors;
use gloo_console::log;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MarkdownViewerProps {
    /// which endpoint to fetch
    pub endpoint: ApiEndpoint,
    /// CSS id on the wrapper `<div>`
    #[prop_or_default]
    pub id: String,
    /// CSS classes on the wrapper `<div>`
    #[prop_or_default]
    pub class: String,
    /// text to show while loading
    #[prop_or("Loading content...".into())]
    pub loading_text: String,
}

#[function_component(MarkdownViewer)]
pub fn markdown_viewer(props: &MarkdownViewerProps) -> Html {
    let content = use_state(String::new);
    let api = props.endpoint.clone();

    {
        let content = content.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                match api.request().send().await {
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

    let inner = Html::from_html_unchecked(AttrValue::from((*content).clone()));

    html! {
        <div id={props.id.clone()} class={props.class.clone()} tabindex="0">
            if content.is_empty() {
                <div class="loading">{ &props.loading_text }</div>
            } else {
                <div>{ inner }</div>
            }
        </div>
    }
}

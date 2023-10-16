use shared::utils::img_to_base64::image_to_base64;
use shared::utils::root_dir::root_dir;
use std::path::Path;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct AppProps {
    pub references: Vec<shared::types::map_reference::MapReference>,
}

#[function_component]
pub fn App(props: &AppProps) -> Html {
    let references = props
        .references
        .iter()
        .map(|reference| {
            let name = String::from(&reference.name);
            let download_url = format!("https://dd2vtt.com/download/{}", reference.hash);
            let src = format!(
                "data:image/png;base64,{}",
                image_to_base64(&Path::new(&root_dir().unwrap()).join(&reference.path.replace(".dd2vtt", ".preview.png")))
            );

            html! {
                <>
                    <div id={"map-asset-view"} class="card">
                        <h1>{name}</h1>
                        <img {src} />
                        <a target={"_blank"} href={download_url}>{"Download DD2VTT File"}</a>
                    </div>
                </>
            }
        })
        .collect::<Html>();

    html! {
        <>
            <div id={"map-asset-vcc"}>
                {references}
            </div>
        </>
    }
}

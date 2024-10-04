use yew::{function_component, html, Html};

const CATALOG_CONTENT: &str = include_str!("../../../../dist/assets/catalog.html");

#[function_component(Catalog)]
pub fn catalog() -> Html {
    html! {
        <div id="catalog">
            { Html::from_html_unchecked(CATALOG_CONTENT.into()) }
        </div>
    }
}

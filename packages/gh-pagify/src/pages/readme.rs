use gloo_utils::document;
use pulldown_cmark::{html as cmark_html, Options, Parser};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::{function_component, html, Html};

const README_CONTENT: &str = include_str!("../../../../README.md");

#[function_component(ReadMe)]
pub fn readme() -> Html {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(README_CONTENT, options);
    let mut html_output = String::new();
    cmark_html::push_html(&mut html_output, parser);

    let div = document().create_element("div").unwrap_or_else(|_| {
        let fallback = document().create_element("div").unwrap();
        fallback.set_inner_html("Error creating README element.");
        fallback
    });
    div.set_inner_html(&html_output);
    div.set_id("readme");
    div.set_class_name("card");

    if let Some(html_element) = div.dyn_ref::<HtmlElement>() {
        html_element.set_tab_index(0); // Make the readme section focusable for better accessibility.
    }

    // Convert the element to Html for rendering
    html! {
        <>{ Html::VRef(div.into()) }</>
    }
}

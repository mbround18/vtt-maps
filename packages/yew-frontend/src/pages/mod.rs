use yew_router::prelude::*;

mod catalog;
mod map_detail;
mod not_found;
// mod readme;
mod markdown_viewer;

pub use catalog::Catalog;
pub use map_detail::MapDetail;
pub use not_found::NotFound;
// pub use readme::ReadMe;
pub use markdown_viewer::MarkdownViewer;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    ReadMe,
    #[at("/LICENSE")]
    License,
    #[at("/catalog")]
    Catalog,
    #[at("/maps/:id")]
    MapDetail { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

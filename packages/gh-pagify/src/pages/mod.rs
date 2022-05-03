use yew_router::prelude::*;

mod catalog;
mod map_asset_view;
mod not_found;
mod readme;

pub use catalog::Catalog;
pub use map_asset_view::MapAssetView;
pub use not_found::NotFound;
pub use readme::ReadMe;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    ReadMe,
    #[at("/catalog")]
    Catalog,
    #[at("/catalog/:id")]
    MapAssetView { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

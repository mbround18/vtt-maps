use yew_router::prelude::*;

mod catalog;
mod not_found;
mod readme;

pub use catalog::Catalog;
pub use not_found::NotFound;
pub use readme::ReadMe;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    ReadMe,
    #[at("/catalog")]
    Catalog,
    #[not_found]
    #[at("/404")]
    NotFound,
}

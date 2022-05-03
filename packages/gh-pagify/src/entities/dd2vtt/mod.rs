use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub struct DD2VTT {
    pub image: String
}

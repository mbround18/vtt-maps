use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Coordinates {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct MapResolution {
    pub map_size: Coordinates,
    pub pixels_per_grid: u16,
}

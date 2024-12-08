use serde::{Deserialize, Serialize};

use super::display::AtcDisplay;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcFacility {
    pub name: String,
    pub displays: Vec<AtcDisplay>,
    pub child_facilities: Vec<AtcFacility>
}
use serde::{Deserialize, Serialize};

use super::{eram::EramConfig, stars::StarsConfiguration, tower::TowerCabConfig};


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcFacility {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub child_facilities: Vec<CrcFacility>,
    pub eram_configuration: Option<EramConfig>,
    pub stars_configuration: Option<StarsConfiguration>,
    pub tower_cab_configuration: Option<TowerCabConfig>,
    pub asdex_configuration: Option<TowerCabConfig>,
    pub neighboring_facility_ids: Vec<String>,
    pub non_nas_facility_ids: Vec<String>,
    pub positions: Option<Vec<CrcPosition>>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcPosition {
    pub id: String,
    pub name: String,
    pub starred: Option<bool>,
    pub radio_name: Option<String>,
    pub callsign: Option<String>,
    pub frequency: Option<u32>,
    pub eram_configuration: Option<CrcPositionEramConfig>,
    pub stars_configuration: Option<CrcPositionStarsConfig>,
    pub tranceiver_ids: Option<Vec<String>>,
    pub runway_ids: Option<Vec<String>>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcPositionEramConfig {
    pub sector_id: String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcPositionStarsConfig {
    pub sector_id: String,
    pub subset: u32,
    pub area_id: String,
    pub color_set: String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconCodeBank {
    id: String,
    category: Option<String>,
    priority: Option<String>,
    subset: Option<u32>,
    start: u32,
    end: u32
}
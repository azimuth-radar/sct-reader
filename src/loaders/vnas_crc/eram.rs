use aviation_calc_util::geo::GeoPoint;
use serde::{Deserialize, Serialize};

use super::facility::BeaconCodeBank;


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EramConfig {
    pub nas_id: String,
    pub geo_maps: Vec<EramGeoMap>,
    pub emergency_checklist: Vec<String>,
    pub position_relief_checklist: Vec<String>,
    pub internal_airports: Vec<String>,
    pub beacon_code_banks: Vec<BeaconCodeBank>,
    pub neighboring_stars_configurations: Vec<NeighborStarsConfig>,
    pub reference_fixes: Vec<String>,
    pub asr_sites: Vec<AsrSite>,
    pub conflict_alert_floor: u32,
    pub airport_single_chars: Vec<String>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EramGeoMap {
    pub id: String,
    pub name: String,
    pub label_line_1: String,
    pub label_line_2: String,
    pub filter_menu: Vec<EramFilterMenu>,
    pub bcg_menu: Vec<String>,
    pub video_map_ids: Vec<String>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EramFilterMenu {
    pub id: String,
    pub label_line_1: String,
    pub label_line_2: String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeighborStarsConfig {
    pub id: String,
    pub facility_id: String,
    pub stars_id: String,
    pub single_character_stars_id: Option<String>,
    pub field_e_format: Option<String>,
    pub field_e_letter: Option<String>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrSite {
    pub id: String,
    pub asr_id: String,
    pub location: GeoPoint,
    pub range: u32,
    pub ceiling: u32
}
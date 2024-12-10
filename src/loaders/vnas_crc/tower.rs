use aviation_calc_util::geo::GeoPoint;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TowerCabConfig {
    pub video_map_id: String,
    pub default_rotation: f32,
    pub default_zoom_range: u32,
    pub aircraft_visibility_ceiling: Option<u32>,
    pub tower_location: Option<GeoPoint>
}

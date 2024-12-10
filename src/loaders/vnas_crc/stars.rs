use aviation_calc_util::geo::GeoPoint;
use serde::{Deserialize, Serialize};

use super::facility::BeaconCodeBank;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarsConfiguration {
    pub areas: Vec<StarsArea>,
    pub internal_airports: Vec<String>,
    pub beacon_code_banks: Vec<BeaconCodeBank>,
    pub rpcs: Vec<StarsRpc>,
    pub video_map_ids: Vec<String>,
    pub map_groups: Vec<StarsMapGroup>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarsArea {
    pub id: String,
    pub name: String,
    pub visibility_center: GeoPoint,
    pub surveillance_range: u32,
    pub underlying_airports: Vec<String>,
    pub ssa_airports: Vec<String>,
    pub tower_list_configurations: Vec<StarsTowerListConfig>,
    pub ldb_beacon_codes_inhibited: bool,
    pub pdb_ground_speed_inhibited: bool,
    pub display_requested_alt_in_fdb: bool,
    pub use_vfr_position_symbol: bool,
    pub show_destination_departures: bool,
    pub show_destination_satellite_arrivals: bool,
    pub show_destination_primary_arrivals: bool
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarsTowerListConfig {
    pub id: String,
    pub airport_id: String,
    pub range: u32
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarsRpc {
    pub id: String,
    pub index: u32,
    pub airport_id: String,
    pub position_symbol_tie: String,
    pub position_symbol_stagger: String,
    pub master_runway: StarsRpcRwy,
    pub slave_runway: StarsRpcRwy
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarsRpcRwy {
    pub runway_id: String,
    pub heading_tolerance: f32,
    pub near_side_half_width: f32,
    pub far_side_half_width: f32,
    pub near_side_distance: f32,
    pub region_length: f32,
    pub target_reference_point: GeoPoint,
    pub target_reference_line_heading: f32,
    pub target_reference_line_length: f32,
    pub target_reference_point_altitude: f32,
    pub image_reference_point: GeoPoint,
    pub image_reference_line_heading: f32,
    pub image_reference_line_length: f32,
    pub tie_mode_offset: f32,
    pub descent_point_distance: f32,
    pub above_path_tolerance: f32,
    pub below_path_tolerance: f32,
    pub default_leader_direction: String,
    pub scratchpad_patterns: Vec<String>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarsMapGroup {
    pub id: String,
    pub map_ids: Vec<Option<u32>>,
    pub tcps: Vec<String>
}
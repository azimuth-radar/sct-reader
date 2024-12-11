use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcPosition {
    pub name: String,
    pub radio_name: Option<String>,
    pub callsign: Option<String>,
    pub frequency: Option<(u16, u16)>,
    pub tranceivers: Vec<String>,
    pub display_configs: Vec<PositionDisplayConfig>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionDisplayConfig {
    pub display_type: String
}
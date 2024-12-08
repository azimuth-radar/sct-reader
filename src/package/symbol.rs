use geojson::{Feature, Geometry, Value};
use serde::{Deserialize, Serialize};

use crate::loaders::euroscope::position::{Position, Valid};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcMapSymbol {
    pub name: String,
    pub symbol_type: String,
    pub feature: Feature
}

impl AtcMapSymbol {
    pub fn from_es_position(sector_file_id: String, item_type: String, ident: String, position: Position<Valid>) -> Self {
        let ident = format!("{}_{}_{}", sector_file_id.to_string(), item_type.to_string(), ident.to_string());
                AtcMapSymbol {
                    name: ident.to_string(),
                    symbol_type: item_type.to_string(),
                    feature: Feature {
                        id: None,
                        bbox: None,
                        foreign_members: None,
                        geometry: Some(Geometry::new(Value::Point(
                            vec![position.lon, position.lat]
                        ))),
                        properties: None
                    }
                }

    }
}
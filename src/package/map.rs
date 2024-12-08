use geojson::{Feature, FeatureCollection, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::loaders::euroscope::{line::{ColouredLine, LineGroup}, sector::{LabelGroup, RegionGroup}};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcMap {
    pub name: String,
    pub map_type: String,
    pub features: FeatureCollection
}

impl AtcMap {
    pub fn try_from_es_line_group(sector_file_id: String, item_type: String, value: LineGroup<ColouredLine>) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.lines.len());
        for line in value.lines {
            // Properties
            let mut props_map = Map::new();
            if let Some(line_color) = line.colour {
                props_map.insert("color".to_string(), serde_json::to_value(format!("#{:02X}{:02X}{:02X}", line_color.r, line_color.g, line_color.b))?);
            }

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::LineString(
                    vec![
                        vec![line.line.start.lon, line.line.start.lat],
                        vec![line.line.end.lon, line.line.end.lat]
                    ]
                ))),
                properties: Some(props_map)
            });
        }

        Ok(AtcMap {
            name: name,
            map_type: item_type,
            features: FeatureCollection {
                bbox: None,
                features: features,
                foreign_members: None
            }
        })
    }

    pub fn try_from_es_region_group(sector_file_id: String, item_type: String, value: RegionGroup) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.regions.capacity());
        for region in value.regions {
            // Properties
            let mut props_map = Map::new();
            props_map.insert("color".to_string(), serde_json::to_value(format!("#{:02X}{:02X}{:02X}", region.colour.r, region.colour.g, region.colour.b))?);

            let mut points = region.vertices.iter().map(|vert| vec![vert.lon, vert.lat]).collect::<Vec<Vec<f64>>>();
            if let Some(start_pt) = points.get(0) {
                points.push(start_pt.clone());
            }

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Polygon(
                    vec![points]
                ))),
                properties: Some(props_map)
            });
        }

        Ok(AtcMap {
            name: name,
            map_type: item_type,
            features: FeatureCollection {
                bbox: None,
                features: features,
                foreign_members: None
            }
        })
    }

    pub fn try_from_es_labels_group(sector_file_id: String, item_type: String, value: LabelGroup) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.labels.capacity());
        for label in value.labels {
            // Properties
            let mut props_map = Map::new();
            props_map.insert("textColor".to_string(), serde_json::to_value(format!("#{:02X}{:02X}{:02X}", label.colour.r, label.colour.g, label.colour.b))?);
            props_map.insert("text".to_string(), serde_json::to_value(label.name.to_string())?);
            props_map.insert("showText".to_string(), serde_json::to_value(true)?);

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Point(
                    vec![label.position.lon, label.position.lat]
                ))),
                properties: Some(props_map)
            });
        }

        Ok(AtcMap {
            name: name,
            map_type: item_type,
            features: FeatureCollection {
                bbox: None,
                features: features,
                foreign_members: None
            }
        })
    }
}
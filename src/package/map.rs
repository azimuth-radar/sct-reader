use crate::loaders::vnas_crc::CrcVideoMapRef;
use crate::loaders::{
    ese::FreeTextGroup,
    euroscope::{
        line::{ColouredLine, LineGroup},
        sector::{LabelGroup, RegionGroup},
    },
};
use anyhow::{anyhow, bail, Context};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtcMapData {
    Embedded {features: FeatureCollection},
    ExternalFile {filename: String}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcMap {
    pub name: String,
    pub data: AtcMapData
}

impl AtcMap {
    pub fn try_from_es_line_group(sector_file_id: String, item_type: String, value: LineGroup<ColouredLine>) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.lines.len());
        for line in value.lines {
            // Properties
            let mut props_map = Map::new();
            props_map.insert("itemType".to_string(), serde_json::to_value(&item_type)?);
            if let Some(line_color) = line.colour {
                props_map.insert(
                    "color".to_string(),
                    serde_json::to_value(format!("#{:02X}{:02X}{:02X}", line_color.r, line_color.g, line_color.b))?,
                );
            }

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::LineString(vec![
                    vec![line.line.start.lon, line.line.start.lat],
                    vec![line.line.end.lon, line.line.end.lat],
                ]))),
                properties: Some(props_map),
            });
        }

        Ok(AtcMap {
            name: name,
            data: AtcMapData::Embedded { 
                features: FeatureCollection {
                    bbox: None,
                    features: features,
                    foreign_members: None,
                }
            }
        })
    }

    pub fn try_from_es_region_group(sector_file_id: String, item_type: String, value: RegionGroup) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.regions.capacity());
        for region in value.regions {
            // Properties
            let mut props_map = Map::new();
            props_map.insert("itemType".to_string(), serde_json::to_value(&item_type)?);
            props_map.insert(
                "color".to_string(),
                serde_json::to_value(format!("#{:02X}{:02X}{:02X}", region.colour.r, region.colour.g, region.colour.b))?,
            );

            let mut points = region.vertices.iter().map(|vert| vec![vert.lon, vert.lat]).collect::<Vec<Vec<f64>>>();
            if let Some(start_pt) = points.get(0) {
                points.push(start_pt.clone());
            }

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Polygon(vec![points]))),
                properties: Some(props_map),
            });
        }

        Ok(AtcMap {
            name: name,
            data: AtcMapData::Embedded { 
                features: FeatureCollection {
                    bbox: None,
                    features: features,
                    foreign_members: None,
                }
            }
        })
    }

    pub fn try_from_es_labels_group(sector_file_id: String, item_type: String, value: LabelGroup) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.labels.capacity());
        for label in value.labels {
            // Properties
            let mut props_map = Map::new();
            props_map.insert("itemType".to_string(), serde_json::to_value(&item_type)?);
            props_map.insert(
                "textColor".to_string(),
                serde_json::to_value(format!("#{:02X}{:02X}{:02X}", label.colour.r, label.colour.g, label.colour.b))?,
            );
            props_map.insert("text".to_string(), serde_json::to_value(label.name.to_string())?);
            props_map.insert("showText".to_string(), serde_json::to_value(true)?);

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Point(vec![label.position.lon, label.position.lat]))),
                properties: Some(props_map),
            });
        }

        Ok(AtcMap {
            name: name,
            data: AtcMapData::Embedded { 
                features: FeatureCollection {
                    bbox: None,
                    features: features,
                    foreign_members: None,
                }
            }
        })
    }

    pub fn try_from_es_freetext_group(sector_file_id: String, item_type: String, value: FreeTextGroup) -> anyhow::Result<Self> {
        let name = format!("{}_{}_{}", sector_file_id, item_type, value.name);
        let mut features = Vec::with_capacity(value.entries.capacity());
        for label in value.entries {
            // Properties
            let mut props_map = Map::new();
            props_map.insert("itemType".to_string(), serde_json::to_value(&item_type)?);
            props_map.insert("text".to_string(), serde_json::to_value(label.text.to_string())?);
            props_map.insert("showText".to_string(), serde_json::to_value(true)?);

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Point(vec![label.position.lon, label.position.lat]))),
                properties: Some(props_map),
            });
        }

        Ok(AtcMap {
            name: name,
            data: AtcMapData::Embedded { 
                features: FeatureCollection {
                    bbox: None,
                    features: features,
                    foreign_members: None,
                }
            }
        })
    }

    pub fn try_from_crc_video_map(map_ref: &CrcVideoMapRef, facility_file_path: impl AsRef<Path>, facility_name: String) -> anyhow::Result<AtcMap> {
        // Determine path
        let video_map_path = facility_file_path
            .as_ref()
            .join("..")
            .join("..")
            .join("VideoMaps")
            .join(facility_name.to_string())
            .join(format!("{}.geojson", map_ref.id))
            .canonicalize()?;

        let geojson = GeoJson::from_reader(BufReader::new(File::open(&video_map_path).context(format!(
            "Couldn't open video map at path {}",
            &video_map_path.to_str().unwrap_or_default().to_string()
        ))?))
        .context("Couldn't parse GeoJSON")?;

        if let GeoJson::FeatureCollection(mut features) = geojson {
            let mut new_features = Vec::new();
            let mut default_line_style = "solid".to_string();
            let mut default_line_thickness = 1;
            let mut default_text_opaque = false;
            let mut default_text_size = 1;
            let mut default_text_underline = false;
            let mut default_text_offset = (0, 0);
            let mut default_symbol_style = "".to_string();
            let mut default_symbol_size = 1;

            for feature in &features {
                let properties = feature.properties.clone().unwrap_or_default();
                // Handle Defaults
                if properties
                    .get(&"isLineDefaults".to_string())
                    .unwrap_or(&serde_json::Value::Bool(false))
                    .as_bool()
                    .unwrap_or(false)
                {
                    if let Some(value) = properties.get(&"style".to_string()) {
                        default_line_style = value.as_str().unwrap_or_default().to_lowercase();
                    }
                    if let Some(value) = properties.get(&"thickness".to_string()) {
                        default_line_thickness = value.as_i64().unwrap_or(1) as i32;
                    }
                } else if properties
                    .get(&"isTextDefaults".to_string())
                    .unwrap_or(&serde_json::Value::Bool(false))
                    .as_bool()
                    .unwrap_or(false)
                {
                    if let Some(value) = properties.get(&"size".to_string()) {
                        default_text_size = value.as_i64().unwrap_or(1) as i32;
                    }
                    if let Some(value) = properties.get(&"xOffset".to_string()) {
                        default_text_offset.0 = value.as_i64().unwrap_or(0) as i32;
                    }
                    if let Some(value) = properties.get(&"yOffset".to_string()) {
                        default_text_offset.1 = value.as_i64().unwrap_or(0) as i32;
                    }
                    if let Some(value) = properties.get(&"opaque".to_string()) {
                        default_text_opaque = value.as_bool().unwrap_or(false);
                    }
                    if let Some(value) = properties.get(&"underline".to_string()) {
                        default_text_underline = value.as_bool().unwrap_or(false);
                    }
                } else if properties
                    .get(&"isSymbolDefaults".to_string())
                    .unwrap_or(&serde_json::Value::Bool(false))
                    .as_bool()
                    .unwrap_or(false)
                {
                    if let Some(value) = properties.get(&"style".to_string()) {
                        default_symbol_style = value.as_str().unwrap_or_default().to_lowercase();
                    }
                    if let Some(value) = properties.get(&"size".to_string()) {
                        default_symbol_size = value.as_i64().unwrap_or(1) as i32;
                    }
                } else if let Some(geometry) = &feature.geometry {
                    let mut new_props = serde_json::Map::new();

                    // Set color
                    if let Some(color) = properties.get(&"color".to_string()) {
                        new_props.insert("color".to_string(), color.clone());
                    }

                    if let Some(z_index) = properties.get(&"zIndex".to_string()) {
                        new_props.insert("zIndex".to_string(), z_index.clone());
                    }

                    match geometry.value.type_name() {
                        "Point" => {
                            // Check for text
                            if let Some(text) = properties.get(&"text".to_string()) {
                                new_props.insert(
                                    "size".to_string(),
                                    properties
                                        .get(&"size".to_string())
                                        .cloned()
                                        .unwrap_or(serde_json::to_value(default_text_size)?),
                                );
                                new_props.insert(
                                    "opaque".to_string(),
                                    properties
                                        .get(&"opaque".to_string())
                                        .cloned()
                                        .unwrap_or(serde_json::to_value(default_text_opaque)?),
                                );
                                new_props.insert(
                                    "underline".to_string(),
                                    properties
                                        .get(&"underline".to_string())
                                        .cloned()
                                        .unwrap_or(serde_json::to_value(default_text_underline)?),
                                );
                                new_props.insert(
                                    "size".to_string(),
                                    properties
                                        .get(&"size".to_string())
                                        .cloned()
                                        .unwrap_or(serde_json::to_value(default_text_size)?),
                                );
                                let mut text_str = "".to_string();
                                for line in (&text).as_array().unwrap_or(&Vec::new()) {
                                    if text_str == "" {
                                        text_str = line.as_str().unwrap_or_default().to_string();
                                    } else {
                                        text_str = format!("{}\n{}", &text_str, line.as_str().unwrap_or_default());
                                    }
                                }
                                new_props.insert("text".to_string(), serde_json::to_value(&text_str)?);
                                new_props.insert("showText".to_string(), serde_json::to_value(true)?);
                            } else {
                                new_props.insert(
                                    "style".to_string(),
                                    properties
                                        .get(&"style".to_string())
                                        .cloned()
                                        .unwrap_or(serde_json::to_value(default_symbol_style.to_string())?),
                                );
                                new_props.insert(
                                    "size".to_string(),
                                    properties
                                        .get(&"size".to_string())
                                        .cloned()
                                        .unwrap_or(serde_json::to_value(default_symbol_size)?),
                                );
                                new_props.insert("showSymbol".to_string(), serde_json::to_value(true)?);
                            }
                        }
                        "LineString" => {
                            new_props.insert(
                                "style".to_string(),
                                properties
                                    .get(&"style".to_string())
                                    .cloned()
                                    .unwrap_or(serde_json::to_value(default_line_style.to_string())?),
                            );

                            new_props.insert(
                                "thickness".to_string(),
                                properties
                                    .get(&"thickness".to_string())
                                    .cloned()
                                    .unwrap_or(serde_json::to_value(&default_line_thickness)?),
                            );
                        }
                        &_ => {}
                    };

                    if let Some(asdex_item_type) = properties.get(&"asdex".to_string()) {
                        new_props.insert("itemType".to_string(), asdex_item_type.clone());
                        new_props.remove(&"color".to_string());
                    } else {
                        new_props.insert(
                            "itemType".to_string(),
                            serde_json::to_value(format!("stars-{}", &map_ref.stars_brightness_category))?,
                        );
                    }

                    new_features.push(Feature {
                        bbox: feature.bbox.clone(),
                        geometry: feature.geometry.clone(),
                        id: feature.id.clone(),
                        properties: Some(new_props),
                        foreign_members: feature.foreign_members.clone(),
                    });
                }
            }

            return Ok(AtcMap {
                name: map_ref.name.to_string(),
                data: AtcMapData::Embedded { 
                    features: FeatureCollection {
                        bbox: None,
                        features: new_features,
                        foreign_members: None,
                    }
                }
            });
        }

        Err(anyhow!("No Features found in GeoJSON!"))
    }
}

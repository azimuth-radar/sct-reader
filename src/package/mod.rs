use std::collections::HashMap;

use aviation_calc_util::{geo::{Bearing, GeoPoint}, units::{Angle, Length}};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::loaders::euroscope::{colour::Colour, line::{ColouredLine, LineGroup}, loader::EuroScopeResult, sector::{LabelGroup, RegionGroup}, symbology::SymbologyItemType, EsAsr};


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcScopePackage {
    pub facilities: Vec<AtcFacility>,
    pub maps: HashMap<String, AtcMap>
}

impl TryFrom<EuroScopeResult> for AtcScopePackage {
    fn try_from(value: EuroScopeResult) -> anyhow::Result<Self> {
        let mut maps = HashMap::new();
        let mut new_facility = AtcFacility::default();
        new_facility.name = value.prf_name;

        // Parse "maps"
        for sector in value.sectors {
            // Geo
            for geo in sector.1.geo_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Geo.to_key_string(), geo)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC
            for entry in sector.1.artcc_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC Low
            for entry in sector.1.artcc_low_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccLowBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC High
            for entry in sector.1.artcc_high_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccHighBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Low Airways
            for entry in sector.1.low_airways {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::LowAirways.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // High Airways
            for entry in sector.1.high_airways {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::HighAirways.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // SIDs
            for entry in sector.1.sid_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Sids.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // STARs
            for entry in sector.1.star_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Stars.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Regions
            for entry in sector.1.regions {
                let val = AtcMap::try_from_es_region_group(sector.0.to_string(), SymbologyItemType::Region.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Labels
            for entry in sector.1.labels {
                let val = AtcMap::try_from_es_labels_group(sector.0.to_string(), SymbologyItemType::Label.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }
        }

        // Parse ASRs
        let mut map_defaults = HashMap::new();
        let mut symbol_defaults = HashMap::new();

        for symbol in value.symbology.symbols {
            if matches!(symbol.item_type, SymbologyItemType::Airports | SymbologyItemType::Fixes | SymbologyItemType::Vors | SymbologyItemType::Ndbs) {
                let mut symb_cfg = DisplayDefaultConfig::default();
                let mut name_cfg = DisplayDefaultConfig::default();
                for attr in symbol.defs {
                    if attr.attribute == "name" {
                        name_cfg.color = attr.color;
                        name_cfg.line_style = attr.line_style;
                        name_cfg.line_weight = attr.line_weight;
                        name_cfg.size = attr.size;
                        name_cfg.text_align = attr.text_align;
                    } else {
                        symb_cfg.color = attr.color;
                        symb_cfg.line_style = attr.line_style;
                        symb_cfg.line_weight = attr.line_weight;
                        symb_cfg.size = attr.size;
                        symb_cfg.text_align = attr.text_align;
                    }
                }
                symbol_defaults.insert(symbol.item_type.to_key_string(), (symb_cfg, name_cfg));
            } else if matches!(symbol.item_type, SymbologyItemType::ArtccBoundary | SymbologyItemType::ArtccHighBoundary | SymbologyItemType::ArtccLowBoundary | SymbologyItemType::Geo | SymbologyItemType::LowAirways | SymbologyItemType::HighAirways | SymbologyItemType::Region) {
                let mut cfg = DisplayDefaultConfig::default();

                for attr in symbol.defs {
                    if attr.attribute == "line" {
                        cfg.color = attr.color;
                        cfg.line_style = attr.line_style;
                        cfg.line_weight = attr.line_weight;
                        cfg.size = attr.size;
                        cfg.text_align = attr.text_align;
                    }
                }

                map_defaults.insert(symbol.item_type.to_key_string(), cfg);
            }
        }
        
        for asr in value.asrs {
            let mut disp = AtcDisplay::from_es_asr(value.default_sector_id.to_string(), map_defaults.clone(), symbol_defaults.clone(), asr.1);
            new_facility.displays.push(disp);
        }

        Ok(AtcScopePackage {
            facilities: vec![new_facility],
            maps: maps
        })
    }
    
    type Error = anyhow::Error;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcFacility {
    pub name: String,
    pub displays: Vec<AtcDisplay>,
    pub child_facilities: Vec<AtcFacility>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtcDisplayItem {
    Map{id: String},
    Symbol{symbol_type: String, ident: String, label: bool},
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayDefaultConfig {
    pub color: Colour,
    pub size: f32,
    pub line_weight: u8,
    pub line_style: u8,
    pub text_align: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcDisplay {
    pub name: String,
    pub center: GeoPoint,
    pub screen_height: Length,
    pub rotation: Angle,
    pub display_items: Vec<AtcDisplayItem>,
    pub map_defaults: HashMap<String, DisplayDefaultConfig>,
    pub symbol_defaults:  HashMap<String, (DisplayDefaultConfig, DisplayDefaultConfig)>,
}

impl AtcDisplay {
    fn from_es_asr(default_sector_id: String, map_defaults: HashMap<String, DisplayDefaultConfig>, symbol_defaults: HashMap<String, (DisplayDefaultConfig, DisplayDefaultConfig)>, value: EsAsr) -> Self {
        let mut ret_val = AtcDisplay::default();
        ret_val.name = value.name;
        ret_val.map_defaults = map_defaults;
        ret_val.symbol_defaults = symbol_defaults;

        // Center
        let dist = (value.window_area.1 - value.window_area.0) / 2;
        let bearing = GeoPoint::initial_bearing(&value.window_area.0, &value.window_area.1);
        let mut center = value.window_area.1.clone();
        center.move_by(bearing, dist);
        ret_val.center = center;

        // Screen Height
        let theta = (Bearing::from_radians(0_f64) - bearing) + value.display_rotation;
        ret_val.screen_height = dist * 2 * theta.as_radians().cos().abs();
        ret_val.rotation = value.display_rotation;

        let mut items = Vec::new();

        let sector_id = value.sector_file_id.clone().unwrap_or(default_sector_id.to_string());

        for item in value.display_items {
            if matches!(item.item_type, SymbologyItemType::Airports | SymbologyItemType::Fixes | SymbologyItemType::Ndbs | SymbologyItemType::Vors) {
                items.push(AtcDisplayItem::Symbol { ident: item.name, label: item.attribute == "name", symbol_type: item.item_type.to_key_string() })
            } else if matches!(item.item_type, SymbologyItemType::ArtccBoundary | SymbologyItemType::ArtccHighBoundary | SymbologyItemType::ArtccLowBoundary | SymbologyItemType::Geo | SymbologyItemType::HighAirways | SymbologyItemType::LowAirways | SymbologyItemType::Region | SymbologyItemType::Sids | SymbologyItemType::Stars) {
                items.push(AtcDisplayItem::Map { id: format!("{}_{}_{}", sector_id.to_string(), item.item_type.to_key_string(), item.name) })
            }
        }

        ret_val.display_items = items;

        ret_val
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcMap {
    pub name: String,
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

            features.push(Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Polygon(
                    vec![region.vertices.iter().map(|vert| vec![vert.lon, vert.lat]).collect::<Vec<Vec<f64>>>()]
                ))),
                properties: Some(props_map)
            });
        }

        Ok(AtcMap {
            name: name,
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
            props_map.insert("color".to_string(), serde_json::to_value(format!("#{:02X}{:02X}{:02X}", label.colour.r, label.colour.g, label.colour.b))?);
            props_map.insert("text".to_string(), serde_json::to_value(label.name.to_string())?);

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
            features: FeatureCollection {
                bbox: None,
                features: features,
                foreign_members: None
            }
        })
    }
}
use std::collections::HashMap;
use std::ops::Deref;
use aviation_calc_util::{geo::{Bearing, GeoPoint}, units::{Angle, Length}};
use geojson::{Feature, FeatureCollection, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::loaders::euroscope::{colour::Colour, line::{ColouredLine, LineGroup}, sector::RegionGroup, symbology::{self, SymbologyInfo, SymbologyItemType}, EsAsr};
use crate::loaders::euroscope::partial::SidStarType::Star;
use crate::loaders::vnas_crc::CrcVideoMapRef;
use crate::loaders::vnas_crc::eram::EramConfig;
use crate::loaders::vnas_crc::stars::{StarsArea, StarsConfiguration};
use crate::loaders::vnas_crc::tower::TowerCabConfig;
use super::symbol::SymbolIcon;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtcDisplayItem {
    Map{id: String, visible: bool},
    Symbol{id: String, show_symbol: bool, show_label: bool},
    NavdataItem{symbol_type: String, ident: String, show_symbol: bool, show_label: bool},
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum TextAlign {
    #[default]
    TopLeft = 0,
    CenterLeft = 1,
    BottomLeft = 2,
    TopCenter = 3,
    CenterCenter = 4,
    BottomCenter = 5,
    TopRight = 6,
    CenterRight = 7,
    BottomRight = 8
}

impl From<u8> for TextAlign {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::CenterRight,
            2 => Self::BottomLeft,
            3 => Self::TopCenter,
            4 => Self::CenterCenter,
            5 => Self::BottomCenter,
            6 => Self::TopRight,
            7 => Self::CenterRight,
            8 => Self::BottomRight,
            _ => Self::TopLeft
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayDefaultConfig {
    pub color: Colour,
    pub size: f32,
    pub line_weight: u8,
    pub line_style: String,
    pub text_align: TextAlign,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AtcDisplayBackground {
    #[default]
    Blank,
    Satellite,
    Color(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcDisplayType {
    pub id: String,
    pub map_defaults: HashMap<String, DisplayDefaultConfig>,
    pub symbol_defaults:  HashMap<String, (DisplayDefaultConfig, DisplayDefaultConfig)>,
    pub symbol_icons: HashMap<String, SymbolIcon>,
    pub line_types: HashMap<String, Vec<u8>>,
    pub background: AtcDisplayBackground
}

impl AtcDisplayType {
    pub fn try_from_es_symbology(id: String, symbology: SymbologyInfo) -> anyhow::Result<Self> {
        let mut map_defaults = HashMap::new();
        let mut symbol_defaults = HashMap::new();
        let mut symbol_icons = HashMap::new();
        let mut background = AtcDisplayBackground::Blank;

        for symbol in symbology.symbols {
            if matches!(symbol.item_type, SymbologyItemType::Airports | SymbologyItemType::Fixes | SymbologyItemType::Vors | SymbologyItemType::Ndbs) {
                let mut symb_cfg = DisplayDefaultConfig::default();
                let mut name_cfg = DisplayDefaultConfig::default();
                for attr in symbol.defs {
                    if attr.attribute == "name" {
                        name_cfg.color = attr.color;
                        name_cfg.line_style = Self::es_line_type_to_string(attr.line_style);
                        name_cfg.line_weight = attr.line_weight;
                        name_cfg.size = attr.size;
                        name_cfg.text_align = attr.text_align.into();
                    } else {
                        symb_cfg.color = attr.color;
                        symb_cfg.line_style = Self::es_line_type_to_string(attr.line_style);
                        symb_cfg.line_weight = attr.line_weight;
                        symb_cfg.size = attr.size;
                        symb_cfg.text_align = attr.text_align.into();
                    }
                }
                symbol_defaults.insert(symbol.item_type.to_key_string(), (symb_cfg, name_cfg));
            } else if matches!(symbol.item_type, SymbologyItemType::ArtccBoundary | SymbologyItemType::ArtccHighBoundary | SymbologyItemType::ArtccLowBoundary | SymbologyItemType::Geo | SymbologyItemType::LowAirways | SymbologyItemType::HighAirways | SymbologyItemType::Region | SymbologyItemType::Sids | SymbologyItemType::Stars) {
                let mut cfg = DisplayDefaultConfig::default();

                for attr in symbol.defs {
                    if attr.attribute == "line" {
                        cfg.color = attr.color;
                        cfg.line_style = Self::es_line_type_to_string(attr.line_style);
                        cfg.line_weight = attr.line_weight;
                        cfg.size = attr.size;
                        cfg.text_align = attr.text_align.into();
                    }
                }

                map_defaults.insert(symbol.item_type.to_key_string(), cfg);
            } else if symbol.item_type == SymbologyItemType::Sector {
                for attr in symbol.defs {
                    if attr.attribute == "active sector background" {
                        background = AtcDisplayBackground::Color(format!("#{:02X}{:02X}{:02X}", attr.color.r, attr.color.g, attr.color.b));
                    }
                }
            }
        }

        for icon in symbology.symbol_icons {
            let ret_icon = SymbolIcon::try_from_es_symbol_icon(icon.0, icon.1)?;

            symbol_icons.insert(ret_icon.symbol_type.to_string(), ret_icon);
        }

        Ok(AtcDisplayType {
            id: id.to_string(),
            map_defaults,
            symbol_defaults,
            symbol_icons,
            line_types: Self::get_es_line_types(),
            background: background
        })
    }

    fn get_es_line_types() -> HashMap<String, Vec<u8>> {
        HashMap::from([
            ("solid".to_string(), vec![1]),
            ("dash".to_string(), vec![18, 6]),
            ("dot".to_string(), vec![3, 3]),
            ("dash-dot".to_string(), vec![9, 6, 3, 6]),
            ("dash-dot-dot".to_string(), vec![9, 3, 3, 3, 3, 3])
        ])
    }

    fn es_line_type_to_string(input: u8) -> String {
        match input {
            1 => "dash".to_string(),
            2 => "dot".to_string(),
            3 => "dash-dot".to_string(),
            4 => "dash-dot-dot".to_string(),
            _ => "solid".to_string()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcDisplay {
    pub name: String,
    pub center: GeoPoint,
    pub screen_height: Length,
    pub rotation: Angle,
    pub display_items: Vec<AtcDisplayItem>,
    pub display_type: String    
}

impl AtcDisplay {
    pub fn from_es_asr(default_sector_id: String, display_type: String, value: EsAsr) -> Self {
        let mut ret_val = AtcDisplay::default();
        ret_val.name = value.name;
        ret_val.display_type = display_type;

        // Center
        let dist = (value.window_area.1 - value.window_area.0) / 2;
        let bearing = GeoPoint::initial_bearing(&value.window_area.0, &value.window_area.1);
        let mut center = value.window_area.0.clone();
        center.move_by(bearing, dist);
        ret_val.center = center;

        // Screen Height
        let theta = (Bearing::from_radians(0_f64) - bearing) + value.display_rotation;
        ret_val.screen_height = dist * 2 * theta.as_radians().cos().abs();
        ret_val.rotation = value.display_rotation;

        let mut items = Vec::new();
        let mut symbols_map = HashMap::<String, usize>::new();

        let sector_id = value.sector_file_id.clone().unwrap_or(default_sector_id.to_string());

        let mut loaded_freetexts= HashMap::new();

        for item in value.display_items {
            if matches!(item.item_type, SymbologyItemType::Airports | SymbologyItemType::Fixes | SymbologyItemType::Ndbs | SymbologyItemType::Vors) {
                let ident = format!("{}_{}_{}", sector_id.to_string(), item.item_type.to_key_string(), item.name);

                let symbol_opt = match symbols_map.get_mut(&ident) {
                    Some(symb_index) => items.get_mut(*symb_index),
                    None => {
                        let symb_index = items.len();
                        symbols_map.insert(ident.to_string(), symb_index);
                        items.push(AtcDisplayItem::Symbol { id: ident.to_string(), show_label: false, show_symbol: false});
                        items.get_mut(symb_index)
                    }
                };
                if let Some(symbol) = symbol_opt {
                    if let AtcDisplayItem::Symbol { id, show_symbol, show_label } = symbol {
                        if item.attribute == "symbol" {
                            *show_symbol = true;
                        } else if item.attribute == "name" {
                            *show_label = true;
                        }
                    }                    
                }
            } else if matches!(item.item_type, SymbologyItemType::ArtccBoundary | SymbologyItemType::ArtccHighBoundary | SymbologyItemType::ArtccLowBoundary | SymbologyItemType::Geo | SymbologyItemType::HighAirways | SymbologyItemType::LowAirways | SymbologyItemType::Region | SymbologyItemType::Sids | SymbologyItemType::Stars) {
                items.push(AtcDisplayItem::Map { id: format!("{}_{}_{}", sector_id.to_string(), item.item_type.to_key_string(), item.name), visible: true })
            } else if matches!(item.item_type, SymbologyItemType::Label) {
                if item.attribute == "freetext" {
                    let name_split = item.name.split("\\").collect::<Vec<&str>>();
                    if name_split.len() >= 1 {
                        if !loaded_freetexts.contains_key(&name_split[0].to_string()) {
                            items.push(AtcDisplayItem::Map {id: format!("{}_{}_{}", sector_id.to_string(), item.item_type.to_key_string(), name_split[0].to_string()), visible: true});
                            loaded_freetexts.insert(name_split[0].to_string(), ());
                        }
                    }
                }
            }
        }

        ret_val.display_items = items;

        ret_val
    }

    pub fn from_crc_twr_asdex(display_type: String, display_name: String, twr_cfg: &TowerCabConfig) -> AtcDisplay {
        AtcDisplay {
            name: display_name.to_string(),
            display_type: display_type.to_string(),
            center: twr_cfg.tower_location.unwrap_or_default(),
            screen_height: Length::from_feet(f64::from(twr_cfg.default_zoom_range) * 200_f64),
            rotation: Angle::from_degrees(twr_cfg.default_rotation.into()),
            display_items: vec![AtcDisplayItem::Map {id: twr_cfg.video_map_id.to_string(), visible: true}]
        }
    }

    fn get_tdm_maps_from_crc(video_map_ids: &Vec<String>, map_refs: &HashMap<String, CrcVideoMapRef>) -> (Vec<AtcDisplayItem>, Vec<AtcDisplayItem>) {
        let mut display_items = Vec::new();
        let mut display_items_tdm = Vec::new();

        for video_map_id in video_map_ids {
            // Check for TDM
            if let Some(map_ref) = map_refs.get(video_map_id) {                
                display_items_tdm.push(AtcDisplayItem::Map {id: video_map_id.to_string(), visible: map_ref.stars_always_visible});
                if !map_ref.tdm_only {
                    display_items.push(AtcDisplayItem::Map {id: video_map_id.to_string(), visible: map_ref.stars_always_visible});
                }
            }
        }

        (display_items, display_items_tdm)
    }

    pub fn from_crc_stars(stars_cfg: &StarsConfiguration, map_refs: &HashMap<String, CrcVideoMapRef>) -> Vec<AtcDisplay> {
        let default_area = StarsArea::default();
        let area = stars_cfg.areas.get(0).unwrap_or(&default_area);
        let display_items = Self::get_tdm_maps_from_crc(&stars_cfg.video_map_ids, map_refs);

        vec![
            AtcDisplay {
                name: "STARS".to_string(),
                display_type: "stars".to_string(),
                center: area.visibility_center,
                screen_height: Length::from_nautical_miles(f64::from(area.surveillance_range) * 2_f64),
                rotation: Angle::from_radians(0_f64),
                display_items: display_items.0
            },
            AtcDisplay {
                name: "STARS (Top Down Mode)".to_string(),
                display_type: "stars".to_string(),
                center: area.visibility_center,
                screen_height: Length::from_nautical_miles(f64::from(area.surveillance_range) * 2_f64),
                rotation: Angle::from_radians(0_f64),
                display_items: display_items.1
            }
        ]
    }

    pub fn from_crc_eram(eram_cfg: &EramConfig, map_refs: &HashMap<String, CrcVideoMapRef>) -> Vec<AtcDisplay> {
        let mut displays = Vec::new();
        for geo_map in &eram_cfg.geo_maps {
            let display_items = Self::get_tdm_maps_from_crc(&geo_map.video_map_ids, map_refs);

            displays.push(AtcDisplay {
                name: format!("ERAM {}", geo_map.name),
                display_type: "eram".to_string(),
                center: GeoPoint::default(),
                screen_height: Length::from_nautical_miles(600_f64),
                rotation: Angle::from_radians(0_f64),
                display_items: display_items.0
            });
            displays.push(AtcDisplay {
                name: format!("ERAM {} (Top Down Mode)", geo_map.name),
                display_type: "eram".to_string(),
                center: GeoPoint::default(),
                screen_height: Length::from_nautical_miles(600_f64),
                rotation: Angle::from_radians(0_f64),
                display_items: display_items.1
            });
        }

        displays
    }
}
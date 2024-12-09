use std::collections::HashMap;

use aviation_calc_util::{geo::{Bearing, GeoPoint}, units::{Angle, Length}};
use geojson::{Feature, FeatureCollection, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::loaders::euroscope::{colour::Colour, line::{ColouredLine, LineGroup}, sector::RegionGroup, symbology::SymbologyItemType, EsAsr};

use super::symbol::SymbolIcon;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtcDisplayItem {
    Map{id: String},
    Symbol{id: String, show_symbol: bool, show_label: bool},
    NavdataItem{symbol_type: String, ident: String, show_symbol: bool, show_label: bool},
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum LineStyle {
    #[default]
    Solid = 0,
    Dash = 1,
    Dot = 2,
    DashDot = 3,
    DashDotDot = 4
}

impl From<u8> for LineStyle {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Dash,
            2 => Self::Dot,
            3 => Self::DashDot,
            4 => Self::DashDotDot,
            _ => Self::Solid
        }
    }
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
    pub line_style: LineStyle,
    pub text_align: TextAlign,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcDisplayType {
    pub id: String,
    pub map_defaults: HashMap<String, DisplayDefaultConfig>,
    pub symbol_defaults:  HashMap<String, (DisplayDefaultConfig, DisplayDefaultConfig)>,
    pub symbol_icons: HashMap<String, SymbolIcon>
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
        let mut center = value.window_area.1.clone();
        center.move_by(bearing, dist);
        ret_val.center = center;

        // Screen Height
        let theta = (Bearing::from_radians(0_f64) - bearing) + value.display_rotation;
        ret_val.screen_height = dist * 2 * theta.as_radians().cos().abs();
        ret_val.rotation = value.display_rotation;

        let mut items = Vec::new();
        let mut symbols_map = HashMap::<String, usize>::new();

        let sector_id = value.sector_file_id.clone().unwrap_or(default_sector_id.to_string());

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
                items.push(AtcDisplayItem::Map { id: format!("{}_{}_{}", sector_id.to_string(), item.item_type.to_key_string(), item.name) })
            }
        }

        ret_val.display_items = items;

        ret_val
    }
}
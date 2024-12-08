use std::collections::HashMap;

use aviation_calc_util::{geo::{Bearing, GeoPoint}, units::{Angle, Length}};
use geojson::{Feature, FeatureCollection, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::loaders::euroscope::{colour::Colour, line::{ColouredLine, LineGroup}, sector::RegionGroup, symbology::SymbologyItemType, EsAsr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtcDisplayItem {
    Map{id: String},
    Symbol{id: String, show_symbol: bool, show_label: bool},
    NavdataItem{symbol_type: String, ident: String, show_symbol: bool, show_label: bool},
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
    pub fn from_es_asr(default_sector_id: String, map_defaults: HashMap<String, DisplayDefaultConfig>, symbol_defaults: HashMap<String, (DisplayDefaultConfig, DisplayDefaultConfig)>, value: EsAsr) -> Self {
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
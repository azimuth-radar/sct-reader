use std::default;

use geojson::{Feature, Geometry, Value};
use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::loaders::euroscope::{
    position::{Position, Valid},
    symbology::SymbologyItemType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcMapSymbol {
    pub name: String,
    pub symbol_type: String,
    pub feature: Feature,
}

impl AtcMapSymbol {
    pub fn try_from_es_position(sector_file_id: String, item_type: String, ident: String, position: Position<Valid>) -> anyhow::Result<Self> {
        let id = format!("{}_{}_{}", sector_file_id.to_string(), item_type.to_string(), ident.to_string());
        // Properties
        let mut props_map = Map::new();
        props_map.insert("text".to_string(), serde_json::to_value(ident.to_string())?);

        Ok(AtcMapSymbol {
            name: id.to_string(),
            symbol_type: item_type.to_string(),
            feature: Feature {
                id: None,
                bbox: None,
                foreign_members: None,
                geometry: Some(Geometry::new(Value::Point(vec![position.lon, position.lat]))),
                properties: Some(props_map),
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolDrawItem {
    Line {
        start: (i8, i8),
        end: (i8, i8),
    },
    Polygon(Vec<(i8, i8)>),
    Arc {
        center: (i8, i8),
        radius: i8,
        inner_radius: i8,
        start_angle: i16,
        end_angle: i16,
        fill: bool,
    },
    SetPixel((i8, i8)),
    Ellipse {
        center: (i8, i8),
        radius: (i8, i8),
        inner_radius: (i8, i8),
        rotation: i16,
        start_angle: i16,
        end_angle: i16,
        fill: bool
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolIcon {
    pub symbol_type: String,
    pub draw_items: Vec<SymbolDrawItem>,
}

impl SymbolIcon {
    pub fn try_from_es_symbol_icon(icon_index: u8, icon: Vec<String>) -> anyhow::Result<Self> {
        let name = match icon_index {
            0 => SymbologyItemType::Airports.to_key_string(),
            1 => SymbologyItemType::Ndbs.to_key_string(),
            2 => SymbologyItemType::Vors.to_key_string(),
            3 => SymbologyItemType::Fixes.to_key_string(),
            4 => "aircraft_stby".to_string(),
            5 => "aircraft_prim".to_string(),
            6 => "aircraft_corr_sec_a+c".to_string(),
            7 => "aircraft_corr_sec_s".to_string(),
            8 => "aircraft_corr_prim_a+c".to_string(),
            9 => "aircraft_corr_prim_s".to_string(),
            10 => "aircraft_corr_a+c_ident".to_string(),
            11 => "aircraft_corr_s_ident".to_string(),
            12 => "aircraft_flt_plan".to_string(),
            13 => "aircraft_coast".to_string(),
            14 => "history_dot".to_string(),
            15 => "aircraft_ground".to_string(),
            16 => "aircraft_uncorr_sec_a+c".to_string(),
            17 => "aircraft_uncorr_sec_s".to_string(),
            18 => "aircraft_uncorr_prim_a+c".to_string(),
            19 => "aircraft_uncorr_prim_s".to_string(),
            20 => "aircraft_uncorr_a+c_ident".to_string(),
            21 => "aircraft_uncorr_s_ident".to_string(),
            22 => "ground_vehicle".to_string(),
            23 => "ground_rotorcraft".to_string(),
            _ => "unknown".to_string(),
        };

        let mut draw_items = Vec::new();
        let mut cursor_pos = (0_i8, 0_i8);
        for line in icon {
            let split = line.split(" ").collect::<Vec<&str>>();
            if split.len() > 0 {
                match split[0] {
                    "MOVETO" => cursor_pos = (split[1].parse()?, -split[2].parse()?),
                    "LINETO" => {
                        let end_point = (split[1].parse()?, -split[2].parse::<i8>()?);
                        draw_items.push(SymbolDrawItem::Line {
                            start: cursor_pos,
                            end: end_point,
                        });
                        cursor_pos = end_point;
                    }
                    "SETPIXEL" => {
                        let point = (split[1].parse()?, -split[2].parse::<i8>()?);
                        draw_items.push(SymbolDrawItem::SetPixel(point));
                        cursor_pos = point;
                    }
                    "POLYGON" => {
                        let mut i = 1;
                        let mut points = Vec::new();
                        while i < split.len() - 1 {
                            let point = (split[i].parse::<i8>()?, -split[i + 1].parse::<i8>()?);
                            cursor_pos = point;
                            points.push(point);
                            i += 2;
                        }
                        draw_items.push(SymbolDrawItem::Polygon(points));
                    }
                    "ARC" => {
                        draw_items.push(SymbolDrawItem::Arc {
                            center: (split[1].parse()?, -split[2].parse()?),
                            radius: split[3].parse()?,
                            inner_radius: 0,
                            start_angle: split[4].parse()?,
                            end_angle: split[5].parse()?,
                            fill: false,
                        });
                    }
                    "FILLARC" => {
                        draw_items.push(SymbolDrawItem::Arc {
                            center: (split[1].parse()?, -split[2].parse()?),
                            radius: split[3].parse()?,
                            inner_radius: 0,
                            start_angle: split[4].parse()?,
                            end_angle: split[5].parse()?,
                            fill: true,
                        });
                    }
                    &_ => {}
                }
            }
        }

        Ok(SymbolIcon {
            symbol_type: name.to_string(),
            draw_items: draw_items,
        })
    }
}

use std::{fs::File, io::{BufRead, BufReader}, path::Path};

use aviation_calc_util::{geo::GeoPoint, units::Angle};

use super::symbology::SymbologyItemType;


#[derive(Debug, Clone, Default)]
pub struct EsAsr {
    pub name: String,
    pub display_type_name: String,
    pub display_type_need_radar_content: bool,
    pub display_type_geo_reference: bool,
    pub sector_file_id: Option<String>,
    pub sector_title: String,
    pub display_items: Vec<DisplayItem>,
    pub show_c: bool,
    pub shows_b: bool,
    pub below: i32,
    pub above: i32,
    pub leader: i32,
    pub show_leader: bool,
    pub turn_leader: bool,
    pub history_dots: u32,
    pub simulation_mode: u32,
    pub disable_panning: bool,
    pub disable_zooming: bool,
    pub display_rotation: Angle,
    pub window_area: (GeoPoint, GeoPoint)
}

impl EsAsr {
    pub fn try_from_asr_file(asr_file: impl AsRef<Path>) -> anyhow::Result<(Self, String)> {
        let file_reader = BufReader::new(File::open(&asr_file)?);
        let mut ret_val = Self::default();
        let mut sector_file = "".to_string();

        for line in file_reader.lines() {
            if let Ok(line_str) = line {
                let items = line_str.split(":").collect::<Vec<&str>>();

                if items.len() > 0 {
                    match items[0].to_lowercase().as_str() {
                        "displaytypename" => ret_val.display_type_name = items[1].to_string(),
                        "displaytypeneedradarcontent" => ret_val.display_type_need_radar_content = items[1].parse::<u8>()? != 0,
                        "displaytypegeoreferenced" => ret_val.display_type_geo_reference = items[1].parse::<u8>()? != 0,
                        "sectorfile" => sector_file = items[1].to_string(),
                        "sectortitle" => ret_val.sector_title = items[1].to_string(),
                        "showc" => ret_val.show_c = items[1].parse::<u8>()? != 0,
                        "showsb" => ret_val.shows_b = items[1].parse::<u8>()? != 0,
                        "below" => ret_val.below = items[1].parse()?,
                        "above" => ret_val.above = items[1].parse()?,
                        "leader" => ret_val.leader = items[1].parse()?,
                        "showleader" => ret_val.show_leader = items[1].parse::<u8>()? != 0,
                        "turnleader" => ret_val.turn_leader = items[1].parse::<u8>()? != 0,
                        "history_dots" => ret_val.history_dots = items[1].parse()?,
                        "simulation_mode" => ret_val.simulation_mode = items[1].parse()?,
                        "disablepanning" => ret_val.disable_panning = items[1].parse::<u8>()? != 0,
                        "disablezooming" => ret_val.disable_zooming = items[1].parse::<u8>()? != 0,
                        "displayrotation" => ret_val.display_rotation = Angle::from_degrees(items[1].parse()?),
                        "windowarea" => ret_val.window_area = (GeoPoint::from_degs_and_ft(items[1].parse()?, items[2].parse()?, 0_f64), GeoPoint::from_degs_and_ft(items[3].parse()?, items[4].parse()?, 0_f64)),
                        &_ => {
                            if let Ok(symbol_type) = items[0].try_into() {
                                ret_val.display_items.push(DisplayItem {
                                    item_type: symbol_type,
                                    name: items[1].to_string(),
                                    attribute: items[2].to_string()
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok((ret_val, sector_file.to_string()))
    }
}


#[derive(Debug, Clone, Default)]
pub struct DisplayItem {
    pub item_type: SymbologyItemType,
    pub name: String,
    pub attribute: String
}
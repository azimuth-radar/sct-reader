use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, path::Path};

use super::colour::Colour;

#[derive(Debug, Clone, Default)]
pub enum SymbologyItemType {
    Airports,
    LowAirways,
    HighAirways,
    Fixes,
    Sids,
    Stars,
    ArtccHighBoundary,
    ArtccBoundary,
    ArtccLowBoundary,
    Geo,
    Vors,
    Ndbs,
    Runways,
    Datablock,
    Controller,
    Metar,
    #[default]
    Other,
    Transitions,
    Chat,
    Sector,
    GroundNetwork
}

impl From<&str> for SymbologyItemType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "airports" => Self::Airports,
            "low airways" => Self::LowAirways,
            "high airways" => Self::HighAirways,
            "fixes" => Self::Fixes,
            "sids" => Self::Sids,
            "stars" => Self::Stars,
            "artcc high boundary" => Self::ArtccHighBoundary,
            "artcc low boundary" => Self::ArtccLowBoundary,
            "artcc boundary" => Self::ArtccBoundary,
            "geo" => Self::Geo,
            "vors" => Self::Vors,
            "ndbs" => Self::Ndbs,
            "runways" => Self::Runways,
            "datablock" => Self::Datablock,
            "controller" => Self::Controller,
            "metar" => Self::Metar,
            "transitions" => Self::Transitions,
            "chat" => Self::Chat,
            "ground network" => Self::GroundNetwork,
            "sector" => Self::Sector,
            &_ => Self::Other
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SymbologyItem {
    pub item_type: SymbologyItemType,
    pub defs: Vec<SymbologyAttribute>
}

#[derive(Debug, Clone, Default)]
pub struct SymbologyAttribute {
    pub attribute: String,
    pub color: Colour,
    pub size: f32,
    pub line_weight: u8,
    pub line_style: u8,
    pub text_align: u8,
}

#[derive(Debug)]
pub struct SymbologyInfo {
    pub symbols: Vec<SymbologyItem>,
    pub clipping_area: u8
}

impl SymbologyInfo {
    pub fn try_from_file(symbology_file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_reader = BufReader::new(File::open(&symbology_file)?);
        let mut clip_area = 5_u8;
        let mut symbols: HashMap<String, SymbologyItem> = HashMap::new();

        for line in file_reader.lines() {
            if let Ok(line_str) = line {
                let items = line_str.split(":").collect::<Vec<&str>>();

                if items.len() > 0 {
                    match items[0] {
                        "m_ClipArea" => clip_area = items[1].parse()?,
                        "SYMBOLOGY" => {},
                        "SYMBOLSIZE" => {},
                        "SYMBOL" => {},
                        "SYMBOLITEM" => {},
                        "END" => {},
                        &_ => {
                            // Create Symbol Def
                            let symbol_def = SymbologyAttribute {
                                attribute: items[1].to_string(),
                                color: Colour::from(items[2].parse::<u32>()?),
                                size: items[3].parse()?,
                                line_weight: items[4].parse()?,
                                line_style: items[5].parse()?,
                                text_align: items[6].parse()?
                            };

                            // Update/Push into map
                            if let Some(symbol_item) = symbols.get_mut(items[0]) {
                                symbol_item.defs.push(symbol_def);
                            } else {
                                symbols.insert(items[0].to_string(), SymbologyItem {
                                    item_type: items[0].into(),
                                    defs: vec![symbol_def]
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(SymbologyInfo {
            clipping_area: clip_area,
            symbols: symbols.values().cloned().collect()
        })
    }
}
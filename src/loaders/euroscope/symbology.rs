use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, path::Path};

use super::colour::Colour;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    GroundNetwork,
    Region,
    Label
}

impl SymbologyItemType {
    pub fn to_key_string(&self) -> String {
        match &self {
            SymbologyItemType::Airports => "airports".to_string(),
            SymbologyItemType::LowAirways => "low_airways".to_string(),
            SymbologyItemType::HighAirways => "high_airways".to_string(),
            SymbologyItemType::Fixes => "fixes".to_string(),
            SymbologyItemType::Sids => "sids".to_string(),
            SymbologyItemType::Stars => "stars".to_string(),
            SymbologyItemType::ArtccHighBoundary => "artcc_high".to_string(),
            SymbologyItemType::ArtccBoundary => "artcc".to_string(),
            SymbologyItemType::ArtccLowBoundary => "artcc_low".to_string(),
            SymbologyItemType::Geo => "geo".to_string(),
            SymbologyItemType::Vors => "vors".to_string(),
            SymbologyItemType::Ndbs => "ndbs".to_string(),
            SymbologyItemType::Runways => "runways".to_string(),
            SymbologyItemType::Datablock => "datablock".to_string(),
            SymbologyItemType::Controller => "controller".to_string(),
            SymbologyItemType::Metar => "metar".to_string(),
            SymbologyItemType::Other => "other".to_string(),
            SymbologyItemType::Transitions => "transitions".to_string(),
            SymbologyItemType::Chat => "chat".to_string(),
            SymbologyItemType::Sector => "sector".to_string(),
            SymbologyItemType::GroundNetwork => "ground_network".to_string(),
            SymbologyItemType::Region => "regions".to_string(),
            SymbologyItemType::Label => "free_text".to_string(),
        }
    }
}

impl TryFrom<&str> for SymbologyItemType {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
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
            "other" => Self::Other,
            "regions" => Self::Region,
            "free text" => Self::Label,
            &_ => return Err(())
        })
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

#[derive(Debug, Default)]
pub struct SymbologyInfo {
    pub file_name: String,
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
                    match items[0].to_lowercase().as_str() {
                        "m_cliparea" => clip_area = items[1].parse()?,
                        &_ => {
                            if let Ok(symbol_type) = items[0].try_into() {
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
                                        item_type: symbol_type,
                                        defs: vec![symbol_def]
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SymbologyInfo {
            file_name: symbology_file.as_ref().to_str().unwrap().to_string(),
            clipping_area: clip_area,
            symbols: symbols.values().cloned().collect()
        })
    }
}
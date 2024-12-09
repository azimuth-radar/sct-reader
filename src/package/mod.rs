use std::{collections::HashMap, hash::Hash};

use aviation_calc_util::{geo::{Bearing, GeoPoint}, units::{Angle, Length}};
use display::{AtcDisplay, AtcDisplayType, DisplayDefaultConfig, LineStyle};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use map::AtcMap;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use symbol::{AtcMapSymbol, SymbolIcon};

use crate::loaders::euroscope::{colour::Colour, line::{ColouredLine, LineGroup}, loader::EuroScopeResult, sector::{LabelGroup, RegionGroup}, symbology::SymbologyItemType, EsAsr};

mod facility;
pub use facility::AtcFacility;

pub mod display;
pub mod map;
pub mod symbol;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcScopePackage {
    pub facilities: Vec<AtcFacility>,
    pub maps: HashMap<String, AtcMap>,
    pub symbols: HashMap<String, AtcMapSymbol>,
    pub display_types: HashMap<String, AtcDisplayType>
}

impl TryFrom<EuroScopeResult> for AtcScopePackage {
    fn try_from(value: EuroScopeResult) -> anyhow::Result<Self> {
        let mut maps = HashMap::new();
        let mut symbols = HashMap::new();
        let mut display_types = HashMap::new();
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

            // Airports
            for entry in sector.1.airports {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Airports.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }

            // Fixes
            for entry in sector.1.fixes {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Fixes.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }

            // VORs
            for entry in sector.1.vors {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Vors.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }

            // NDBs
            for entry in sector.1.ndbs {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Ndbs.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }
        }

        // Parse ASRs
        let mut map_defaults = HashMap::new();
        let mut symbol_defaults = HashMap::new();
        let mut symbol_icons = HashMap::new();

        for symbol in value.symbology.symbols {
            if matches!(symbol.item_type, SymbologyItemType::Airports | SymbologyItemType::Fixes | SymbologyItemType::Vors | SymbologyItemType::Ndbs) {
                let mut symb_cfg = DisplayDefaultConfig::default();
                let mut name_cfg = DisplayDefaultConfig::default();
                for attr in symbol.defs {
                    if attr.attribute == "name" {
                        name_cfg.color = attr.color;
                        name_cfg.line_style = attr.line_style.into();
                        name_cfg.line_weight = attr.line_weight;
                        name_cfg.size = attr.size;
                        name_cfg.text_align = attr.text_align.into();
                    } else {
                        symb_cfg.color = attr.color;
                        symb_cfg.line_style = attr.line_style.into();
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
                        cfg.line_style = attr.line_style.into();
                        cfg.line_weight = attr.line_weight;
                        cfg.size = attr.size;
                        cfg.text_align = attr.text_align.into();
                    }
                }

                map_defaults.insert(symbol.item_type.to_key_string(), cfg);
            }
        }

        for icon in value.symbology.symbol_icons {
            let ret_icon = SymbolIcon::try_from_es_symbol_icon(icon.0, icon.1)?;

            symbol_icons.insert(ret_icon.symbol_type.to_string(), ret_icon);
        }

        display_types.insert(value.prf_file.to_string(), AtcDisplayType {
            id: value.prf_file.to_string(),
            map_defaults,
            symbol_defaults,
            symbol_icons
        });
        
        for asr in value.asrs {
            let mut disp = AtcDisplay::from_es_asr(value.default_sector_id.to_string(), value.prf_file.to_string(), asr.1);
            new_facility.displays.push(disp);
        }

        Ok(AtcScopePackage {
            facilities: vec![new_facility],
            symbols: symbols,
            maps: maps,
            display_types
        })
    }
    
    type Error = anyhow::Error;
}
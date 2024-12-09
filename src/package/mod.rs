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
            for geo in sector.1.0.geo_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Geo.to_key_string(), geo)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC
            for entry in sector.1.0.artcc_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC Low
            for entry in sector.1.0.artcc_low_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccLowBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC High
            for entry in sector.1.0.artcc_high_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccHighBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Low Airways
            for entry in sector.1.0.low_airways {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::LowAirways.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // High Airways
            for entry in sector.1.0.high_airways {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::HighAirways.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // SIDs
            for entry in sector.1.0.sid_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Sids.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // STARs
            for entry in sector.1.0.star_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Stars.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Regions
            for entry in sector.1.0.regions {
                let val = AtcMap::try_from_es_region_group(sector.0.to_string(), SymbologyItemType::Region.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Labels
            for entry in sector.1.0.labels {
                let val = AtcMap::try_from_es_labels_group(sector.0.to_string(), SymbologyItemType::Label.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ESE
            if let Some(ese_file) = sector.1.1 {
                for entry in ese_file.free_text {
                    let val = AtcMap::try_from_es_freetext_group(sector.0.to_string(), SymbologyItemType::Label.to_key_string(), entry)?;

                    maps.insert(val.name.to_string(), val);
                }
            }

            // Airports
            for entry in sector.1.0.airports {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Airports.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }

            // Fixes
            for entry in sector.1.0.fixes {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Fixes.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }

            // VORs
            for entry in sector.1.0.vors {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Vors.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }

            // NDBs
            for entry in sector.1.0.ndbs {
                let val = AtcMapSymbol::try_from_es_position(sector.0.to_string(), SymbologyItemType::Ndbs.to_key_string(), entry.identifier, entry.position)?;
                symbols.insert(val.name.to_string(), val);
            }
        }

        // Parse symbology
        display_types.insert(value.prf_file.to_string(), AtcDisplayType::try_from_es_symbology(value.prf_file.to_string(), value.symbology)?);
        
        // Parse ASRs
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
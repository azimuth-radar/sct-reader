use anyhow::Context;
use aviation_calc_util::{
    geo::{Bearing, GeoPoint},
    units::{Angle, Length},
};
use display::{AtcDisplay, AtcDisplayType, DisplayDefaultConfig, LineStyle};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use map::AtcMap;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::{collections::HashMap, hash::Hash};
use symbol::{AtcMapSymbol, SymbolIcon};

use crate::loaders::euroscope::{
    colour::Colour,
    line::{ColouredLine, LineGroup},
    loader::EuroScopeResult,
    sector::{LabelGroup, RegionGroup},
    symbology::SymbologyItemType,
    EsAsr,
};

mod facility;
use crate::loaders::vnas_crc::{CrcPackage, CrcVideoMapRef};
use crate::package::display::AtcDisplayItem;
pub use facility::AtcFacility;

pub mod display;
pub mod map;
pub mod symbol;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcScopePackage {
    pub facilities: Vec<AtcFacility>,
    pub maps: HashMap<String, AtcMap>,
    pub symbols: HashMap<String, AtcMapSymbol>,
    pub display_types: HashMap<String, AtcDisplayType>,
}

impl TryFrom<EuroScopeResult> for AtcScopePackage {
    type Error = anyhow::Error;

    fn try_from(value: EuroScopeResult) -> anyhow::Result<Self> {
        let mut maps = HashMap::new();
        let mut symbols = HashMap::new();
        let mut display_types = HashMap::new();
        let mut facilities = Vec::new();

        // Parse "maps"
        for sector in value.sectors {
            // Geo
            for geo in sector.1 .0.geo_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Geo.to_key_string(), geo)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC
            for entry in sector.1 .0.artcc_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC Low
            for entry in sector.1 .0.artcc_low_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccLowBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ARTCC High
            for entry in sector.1 .0.artcc_high_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::ArtccHighBoundary.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Low Airways
            for entry in sector.1 .0.low_airways {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::LowAirways.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // High Airways
            for entry in sector.1 .0.high_airways {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::HighAirways.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // SIDs
            for entry in sector.1 .0.sid_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Sids.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // STARs
            for entry in sector.1 .0.star_entries {
                let val = AtcMap::try_from_es_line_group(sector.0.to_string(), SymbologyItemType::Stars.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Regions
            for entry in sector.1 .0.regions {
                let val = AtcMap::try_from_es_region_group(sector.0.to_string(), SymbologyItemType::Region.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // Labels
            for entry in sector.1 .0.labels {
                let val = AtcMap::try_from_es_labels_group(sector.0.to_string(), SymbologyItemType::Label.to_key_string(), entry)?;

                maps.insert(val.name.to_string(), val);
            }

            // ESE
            if let Some(ese_file) = sector.1 .1 {
                for entry in ese_file.free_text {
                    let val = AtcMap::try_from_es_freetext_group(sector.0.to_string(), SymbologyItemType::Label.to_key_string(), entry)?;

                    maps.insert(val.name.to_string(), val);
                }
            }

            // Airports
            for entry in sector.1 .0.airports {
                let val = AtcMapSymbol::try_from_es_position(
                    sector.0.to_string(),
                    SymbologyItemType::Airports.to_key_string(),
                    entry.identifier,
                    entry.position,
                )?;
                symbols.insert(val.name.to_string(), val);
            }

            // Fixes
            for entry in sector.1 .0.fixes {
                let val = AtcMapSymbol::try_from_es_position(
                    sector.0.to_string(),
                    SymbologyItemType::Fixes.to_key_string(),
                    entry.identifier,
                    entry.position,
                )?;
                symbols.insert(val.name.to_string(), val);
            }

            // VORs
            for entry in sector.1 .0.vors {
                let val = AtcMapSymbol::try_from_es_position(
                    sector.0.to_string(),
                    SymbologyItemType::Vors.to_key_string(),
                    entry.identifier,
                    entry.position,
                )?;
                symbols.insert(val.name.to_string(), val);
            }

            // NDBs
            for entry in sector.1 .0.ndbs {
                let val = AtcMapSymbol::try_from_es_position(
                    sector.0.to_string(),
                    SymbologyItemType::Ndbs.to_key_string(),
                    entry.identifier,
                    entry.position,
                )?;
                symbols.insert(val.name.to_string(), val);
            }
        }

        for prf in value.profiles {
            let mut facility = AtcFacility::default();
            facility.name = prf.prf_name;

            // Parse symbology
            display_types.insert(
                prf.prf_file.to_string(),
                AtcDisplayType::try_from_es_symbology(prf.prf_file.to_string(), prf.symbology)?,
            );

            // Parse ASRs
            for asr in prf.asrs {
                let mut disp = AtcDisplay::from_es_asr(prf.default_sector_id.to_string(), prf.prf_file.to_string(), asr.1);
                facility.displays.push(disp);
            }

            facilities.push(facility);
        }

        Ok(AtcScopePackage {
            facilities: facilities,
            symbols: symbols,
            maps: maps,
            display_types,
        })
    }
}

impl TryFrom<&CrcPackage> for AtcScopePackage {
    type Error = anyhow::Error;

    fn try_from(value: &CrcPackage) -> Result<Self, Self::Error> {
        let mut package = AtcScopePackage::default();
        let mut maps_map: HashMap<String, CrcVideoMapRef> = HashMap::new();

        // Process maps
        for map_ref in &value.video_maps {
            package.maps.insert(
                map_ref.id.to_string(),
                AtcMap::try_from_crc_video_map(map_ref, &value.file_path, value.id.to_string())?,
            );
            maps_map.insert(map_ref.id.to_string(), map_ref.clone());
        }

        // Process facility
        package.facilities.push(AtcFacility::try_from_crc(&value.facility, &maps_map)?);

        package.display_types.insert(
            "eram".to_string(),
            AtcDisplayType {
                id: "eram".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
            },
        );

        package.display_types.insert(
            "stars".to_string(),
            AtcDisplayType {
                id: "stars".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
            },
        );

        package.display_types.insert(
            "asdex".to_string(),
            AtcDisplayType {
                id: "asdex".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
            },
        );

        package.display_types.insert(
            "twrcab".to_string(),
            AtcDisplayType {
                id: "twrcab".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
            },
        );

        Ok(package)
    }
}

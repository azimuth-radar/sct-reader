use anyhow::Context;
use aviation_calc_util::{
    geo::{Bearing, GeoPoint},
    units::{Angle, Length},
};
use display::{AtcDisplay, AtcDisplayBackground, AtcDisplayType, DisplayDefaultConfig};
use flate2::{read::GzDecoder, write::GzEncoder, Compression, GzBuilder};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use map::{AtcMap, AtcMapData};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use tar::{Archive, Header};
use uuid::Uuid;
use std::{collections::HashMap, env, fmt::format, fs::{self, File}, hash::Hash, io::{BufReader, BufWriter, Read, Write}, path::{Path, PathBuf}};
use symbol::{AtcMapSymbol, SymbolDrawItem, SymbolIcon};

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
pub mod position;
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

        // ERAM Symbols
        package.display_types.insert(
            "eram".to_string(),
            AtcDisplayType {
                id: "eram".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Self::get_eram_symbols(),
                line_types: Self::get_eram_lines(),
                background: AtcDisplayBackground::Color("#000000".to_string()),
            },
        );

        package.display_types.insert(
            "stars".to_string(),
            AtcDisplayType {
                id: "stars".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
                line_types: Default::default(),
                background: AtcDisplayBackground::Color("#000000".to_string()),
            },
        );

        package.display_types.insert(
            "asdex-day".to_string(),
            AtcDisplayType {
                id: "asdex-day".to_string(),
                map_defaults: HashMap::from([
                    (
                        "taxiway".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 45, g: 45, b: 45 },
                            ..Default::default()
                        },
                    ),
                    (
                        "apron".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 70, g: 70, b: 70 },
                            ..Default::default()
                        },
                    ),
                    (
                        "structure".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 96, g: 96, b: 96 },
                            ..Default::default()
                        },
                    ),
                    (
                        "runway".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 0, g: 0, b: 0 },
                            ..Default::default()
                        },
                    ),
                ]),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
                line_types: Default::default(),
                background: AtcDisplayBackground::Color("#005C73".to_string()),
            },
        );

        package.display_types.insert(
            "asdex-night".to_string(),
            AtcDisplayType {
                id: "asdex-night".to_string(),
                map_defaults: HashMap::from([
                    (
                        "taxiway".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 16, g: 37, b: 76 },
                            ..Default::default()
                        },
                    ),
                    (
                        "apron".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 17, g: 52, b: 93 },
                            ..Default::default()
                        },
                    ),
                    (
                        "structure".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 32, g: 60, b: 98 },
                            ..Default::default()
                        },
                    ),
                    (
                        "runway".to_string(),
                        DisplayDefaultConfig {
                            color: Colour { r: 0, g: 0, b: 0 },
                            ..Default::default()
                        },
                    ),
                ]),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
                line_types: Default::default(),
                background: AtcDisplayBackground::Color("#393939".to_string()),
            },
        );

        package.display_types.insert(
            "twrcab".to_string(),
            AtcDisplayType {
                id: "twrcab".to_string(),
                map_defaults: Default::default(),
                symbol_defaults: Default::default(),
                symbol_icons: Default::default(),
                line_types: Default::default(),
                background: display::AtcDisplayBackground::Satellite,
            },
        );

        Ok(package)
    }
}

impl AtcScopePackage {
    fn get_eram_symbols() -> HashMap<String, SymbolIcon> {
        HashMap::from([
            (
                "vor".to_string(),
                SymbolIcon {
                    symbol_type: "vor".to_string(),
                    draw_items: vec![SymbolDrawItem::Ellipse {
                        center: (0, 0),
                        radius: (3, 5),
                        inner_radius: (0, 0),
                        rotation: 0,
                        start_angle: 0,
                        end_angle: 360,
                        fill: false,
                    }],
                },
            ),
            (
                "ndb".to_string(),
                SymbolIcon {
                    symbol_type: "ndb".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Ellipse {
                            center: (0, 0),
                            radius: (3, 5),
                            inner_radius: (0, 0),
                            rotation: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: false,
                        },
                        SymbolDrawItem::Line {
                            start: (-4, -7),
                            end: (4, 7),
                        },
                    ],
                },
            ),
            (
                "obstruction1".to_string(),
                SymbolIcon {
                    symbol_type: "obstruction1".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line {
                            start: (-3, -6),
                            end: (3, 6),
                        },
                        SymbolDrawItem::Line {
                            start: (-3, 6),
                            end: (3, -6),
                        },
                        SymbolDrawItem::Line { start: (-3, 7), end: (3, 7) },
                    ],
                },
            ),
            (
                "obstruction2".to_string(),
                SymbolIcon {
                    symbol_type: "obstruction2".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line {
                            start: (-5, -6),
                            end: (0, 6),
                        },
                        SymbolDrawItem::Line { start: (0, 6), end: (5, -6) },
                        SymbolDrawItem::Arc {
                            center: (0, -4),
                            radius: 2,
                            inner_radius: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: true,
                        },
                    ],
                },
            ),
            (
                "heliport".to_string(),
                SymbolIcon {
                    symbol_type: "heliport".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 6,
                            inner_radius: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: false,
                        },
                        SymbolDrawItem::Line { start: (-2, 0), end: (2, 0) },
                        SymbolDrawItem::Line {
                            start: (-2, 2),
                            end: (-2, -2),
                        },
                        SymbolDrawItem::Line { start: (2, 2), end: (2, -2) },
                    ],
                },
            ),
            (
                "nuclear".to_string(),
                SymbolIcon {
                    symbol_type: "nuclear".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (-1, 0), end: (1, 0) },
                        SymbolDrawItem::Line { start: (0, 1), end: (0, -1) },
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 6,
                            inner_radius: 3,
                            start_angle: 180,
                            end_angle: 240,
                            fill: true,
                        },
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 6,
                            inner_radius: 3,
                            start_angle: 300,
                            end_angle: 360,
                            fill: true,
                        },
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 6,
                            inner_radius: 3,
                            start_angle: 60,
                            end_angle: 120,
                            fill: true,
                        },
                    ],
                },
            ),
            (
                "emergencyairport".to_string(),
                SymbolIcon {
                    symbol_type: "emergencyairport".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (-2, 2), end: (2, 2) },
                        SymbolDrawItem::Line { start: (2, 2), end: (2, -2) },
                        SymbolDrawItem::Line {
                            start: (2, -2),
                            end: (-2, -2),
                        },
                        SymbolDrawItem::Line {
                            start: (-2, -2),
                            end: (-2, 2),
                        },
                        SymbolDrawItem::Line {
                            start: (-5, 5),
                            end: (-2, 2),
                        },
                        SymbolDrawItem::Line {
                            start: (2, -2),
                            end: (5, -5),
                        },
                    ],
                },
            ),
            (
                "radar".to_string(),
                SymbolIcon {
                    symbol_type: "radar".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (0, 3), end: (0, -3) },
                        SymbolDrawItem::Line {
                            start: (0, 3),
                            end: (-4, -2),
                        },
                        SymbolDrawItem::Line { start: (0, -3), end: (4, 2) },
                        SymbolDrawItem::Arc {
                            center: (1, 1),
                            radius: 7,
                            inner_radius: 0,
                            start_angle: 90,
                            end_angle: 180,
                            fill: false,
                        },
                    ],
                },
            ),
            (
                "iaf".to_string(),
                SymbolIcon {
                    symbol_type: "iaf".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (-4, 0), end: (4, 0) },
                        SymbolDrawItem::Line { start: (0, 4), end: (0, -4) },
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 4,
                            inner_radius: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: false,
                        },
                    ],
                },
            ),
            (
                "rnavonlywaypoint".to_string(),
                SymbolIcon {
                    symbol_type: "rnavonlywaypoint".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 3,
                            inner_radius: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: false,
                        },
                        SymbolDrawItem::Arc {
                            center: (0, 0),
                            radius: 5,
                            inner_radius: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: false,
                        },
                    ],
                },
            ),
            (
                "rnav".to_string(),
                SymbolIcon {
                    symbol_type: "rnav".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (-2, 0), end: (2, 0) },
                        SymbolDrawItem::Line { start: (0, 2), end: (0, -2) },
                        SymbolDrawItem::Line { start: (-2, 2), end: (2, 2) },
                        SymbolDrawItem::Line { start: (2, 2), end: (2, -2) },
                        SymbolDrawItem::Line {
                            start: (2, -2),
                            end: (-2, -2),
                        },
                        SymbolDrawItem::Line {
                            start: (-2, -2),
                            end: (-2, 2),
                        },
                    ],
                },
            ),
            (
                "airwayintersections".to_string(),
                SymbolIcon {
                    symbol_type: "airwayintersections".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line {
                            start: (-3, -3),
                            end: (3, -3),
                        },
                        SymbolDrawItem::Line { start: (3, -3), end: (0, 3) },
                        SymbolDrawItem::Line {
                            start: (0, 3),
                            end: (-3, -3),
                        },
                    ],
                },
            ),
            (
                "otherwaypoints".to_string(),
                SymbolIcon {
                    symbol_type: "otherwaypoints".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (-5, 0), end: (5, 0) },
                        SymbolDrawItem::Line { start: (0, 5), end: (0, -5) },
                    ],
                },
            ),
            (
                "airport".to_string(),
                SymbolIcon {
                    symbol_type: "airport".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line { start: (-2, 2), end: (2, 2) },
                        SymbolDrawItem::Line { start: (2, 2), end: (2, -2) },
                        SymbolDrawItem::Line {
                            start: (2, -2),
                            end: (-2, -2),
                        },
                        SymbolDrawItem::Line {
                            start: (-2, -2),
                            end: (-2, 2),
                        },
                    ],
                },
            ),
            (
                "satelliteairport".to_string(),
                SymbolIcon {
                    symbol_type: "satelliteairport".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Line {
                            start: (-6, -4),
                            end: (-6, 3),
                        },
                        SymbolDrawItem::Line { start: (-6, 3), end: (5, 3) },
                    ],
                },
            ),
            (
                "tacan".to_string(),
                SymbolIcon {
                    symbol_type: "tacan".to_string(),
                    draw_items: vec![
                        SymbolDrawItem::Ellipse {
                            center: (0, 0),
                            radius: (3, 5),
                            inner_radius: (0, 0),
                            rotation: 0,
                            start_angle: 0,
                            end_angle: 360,
                            fill: false,
                        },
                        SymbolDrawItem::Line { start: (-2, 0), end: (2, 0) },
                        SymbolDrawItem::Line { start: (0, 2), end: (0, -2) },
                    ],
                },
            ),
        ])
    }

    fn get_eram_lines() -> HashMap<String, Vec<u8>> {
        HashMap::from([
            ("solid".to_string(), vec![1]),
            ("shortdashed".to_string(), vec![8, 8]),
            ("longdashed".to_string(), vec![16, 16]),
            ("longdashshortdash".to_string(), vec![14, 6, 6, 6]),
        ])
    }

    fn write_json_to_targz<W, T>(tar_builder: &mut tar::Builder<W>, tar_path: impl AsRef<Path>, temp_dir: impl AsRef<Path>, file_name: &str, value: &T) -> anyhow::Result<()>
        where W: Write, T: ?Sized + Serialize {
        // Write file
        let map_file_path = &temp_dir.as_ref().join(&file_name);
        let data = serde_json::to_vec(&value).context("Writing json file.")?;

        // Add to TAR.GZ
        let mut header = Header::new_gnu();
        header.set_mode(0o755);
        header.set_size(data.len() as u64);
        &tar_builder.append_data(
            &mut header,
            &tar_path.as_ref().join(&file_name),
            &*data)
            .context("Creating map tar entry.")?;

        Ok(())
    }

    /// Exports the entire ATC Scope Package to a file.
    /// 
    /// File is tarred and gzipped.
    ///
    /// All video maps are externalized from the main JSON file to allow lazy loading.
    /// These video maps are GeoJSON FeatureCollections
    ///
    /// Format is as follows:
    /// - `<name>.atcpkg`
    ///   - `ScopePackage.json`
    ///   - `maps`
    ///     - `<videomap>.geojson`
    ///     - ...
    pub fn export_to_gzip(&self, file_name: impl AsRef<Path>, maps_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        // Create temp working dir
        let temp_dir = env::temp_dir().join("sct_reader");
        fs::create_dir_all(&temp_dir).context("Creating temp dir.")?;
        
        // Create tar and gz
        let gz_file = File::create(file_name)?;
        let gz_builder = GzBuilder::new().write(BufWriter::new(gz_file), Compression::default());
        let mut tar_builder = tar::Builder::new(gz_builder);

        // Create video maps
        let new_maps = self.maps.iter()
            .map(|(id, map)| {
                match &map.data {
                    AtcMapData::Embedded { features } => {
                        // Generate random uuid for filename
                        let map_uuid = Uuid::new_v4().simple().to_string();

                        // Write file
                        let map_file_name = format!("{}.geojson", &map_uuid);
                        Self::write_json_to_targz(&mut tar_builder, "maps", &temp_dir, &map_file_name, &features)?;

                        // Update Map
                        Ok((
                            id.clone(), 
                            AtcMap {
                                name: map.name.clone(),
                                data: AtcMapData::ExternalFile {
                                    filename: format!("{}.geojson", &map_uuid)
                                }
                            }
                        ))
                    },
                    AtcMapData::ExternalFile { filename } => {
                        // Copy File into archive
                        tar_builder.append_file(
                            Path::new("maps").join(filename),
                            &mut File::open(maps_dir.as_ref().join(filename))?
                        )?;
                        Ok((id.clone(), map.clone()))
                    },
                }
            })
            .collect::<anyhow::Result<HashMap<String, AtcMap>>>()?;

        // Create Package
        let new_package = AtcScopePackage {
            facilities: self.facilities.clone(),
            maps: new_maps,
            symbols: self.symbols.clone(),
            display_types: self.display_types.clone()
        };

        // Save Package json
        Self::write_json_to_targz(&mut tar_builder, "", &temp_dir, "ScopePackage.json", &new_package)?;

        // Finish writing tar
        tar_builder.finish().context("Writing TAR file.")?;

        Ok(())
    }

    pub fn import_from_gzip(file_name: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        // Unzip tar
        let tar_gz = File::open(file_name)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(&out_dir)?;

        // Read package
        let package = serde_json::from_reader(BufReader::new(
            File::open(&out_dir.as_ref().join("ScopePackage.json"))?
        ))?;

        Ok(package)
    }

    /// Attempts to lazy load a map.
    /// 
    /// If the map is embedded it will return immediately, otherwise it will load the map from the JSON file.
    /// 
    /// The map in this package will be replaced with the embedded one for performance.
    pub fn try_load_map_data(&mut self, map_id: &str, maps_dir: impl AsRef<Path>) -> anyhow::Result<Option<&AtcMap>> {
        if let Some(map) = self.maps.get_mut(map_id) {
            if let AtcMapData::ExternalFile { filename } = &map.data {
                map.data = AtcMapData::Embedded { 
                    features: serde_json::from_reader(
                        BufReader::new(File::open(maps_dir.as_ref().join(&filename))?)
                    )?
                };
            }

            Ok(Some(map))
        } else {
            Ok(None)
        }
    }
}

use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};

use anyhow::Context;
use directories::UserDirs;

use super::{colour::Colour, reader::SctReader, sector::Sector};

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
    pub defs: Vec<SymbolDef>
}

#[derive(Debug, Clone, Default)]
pub struct SymbolDef {
    pub symbol_type: String,
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

#[derive(Debug)]
pub struct EuroScopeLoader {
    pub prf_file: String,
    pub symbology_file: String,
    pub sector_file: String,
    pub asr_files: Vec<(String, String)>,
    pub sector: Option<Sector>,
    pub symbology: Option<SymbologyInfo>
}

impl EuroScopeLoader {
    pub fn try_new_from_prf(prf_file: impl AsRef<Path>) -> anyhow::Result<EuroScopeLoader> {
        // Vars
        let mut symbology_file = "".to_string();
        let mut sector_file = "".to_string();
        let mut asrs: Vec<(String, String)> = Vec::new();

        // Read PRF File
        let reader = BufReader::new(File::open(&prf_file)?);

        for line in reader.lines() {
            if let Ok(ln) = line {
                let items = ln.split("\t").collect::<Vec<&str>>();
                if items.len() > 0 {
                    match items[0] {
                        "Settings" => {
                            if items.len() >= 3 {
                                match items[1] {
                                    "SettingsfileSYMBOLOGY" => {
                                        symbology_file =
                                            Self::try_convert_es_path(&prf_file, items[2])?
                                                .canonicalize()?
                                                .to_str()
                                                .unwrap()
                                                .to_owned();
                                    }
                                    "sector" => {
                                        sector_file =
                                            Self::try_convert_es_path(&prf_file, items[2])?
                                                .canonicalize()?
                                                .to_str()
                                                .unwrap()
                                                .to_owned();
                                    }
                                    &_ => {}
                                }
                            }
                        }
                        "ASRFastKeys" => asrs.push((
                            items[1].to_owned(),
                            Self::try_convert_es_path(&prf_file, items[2])?
                                .canonicalize()?
                                .to_str()
                                .unwrap()
                                .to_owned(),
                        )),
                        &_ => {}
                    }
                }
            }
        }

        Ok(EuroScopeLoader {
            prf_file: prf_file
                .as_ref()
                .canonicalize()?
                .to_str()
                .unwrap()
                .to_string(),
            symbology_file: symbology_file.to_string(),
            sector_file: sector_file.to_string(),
            asr_files: asrs,
            sector: None,
            symbology: None
        })
    }

    fn load_symbology(&mut self) -> anyhow::Result<()> {
        let file_reader = BufReader::new(File::open(&self.symbology_file)?);
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
                            let symbol_def = SymbolDef {
                                symbol_type: items[1].to_string(),
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

        self.symbology = Some(SymbologyInfo {
            clipping_area: clip_area,
            symbols: symbols.values().cloned().collect()
        });

        Ok(())
    }

    pub fn process_data(&mut self) -> anyhow::Result<()> {
        self.load_symbology();

        // Load Sector File
        let sct_reader = SctReader::new(BufReader::new(File::open(&self.sector_file)?));
        let sct_result = sct_reader.try_read()?;
        self.sector = Some(sct_result);

        Ok(())
    }

    pub fn try_convert_es_path(
        prf_file_path: impl AsRef<Path>,
        es_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let split_es_path = es_path.split("\\").collect::<Vec<&str>>();
        let mut new_es_path = PathBuf::from("");
        for item in &split_es_path {
            new_es_path.push(item);
        }

        // Relative to PRF folder
        if split_es_path.len() > 0 && split_es_path[0] == "" {
            return Ok(prf_file_path
                .as_ref()
                .parent()
                .context("Could not get parent dir of prf file!")?
                .join(new_es_path));
        }

        Ok(UserDirs::new()
            .context("Failed to get User Directories")?
            .document_dir()
            .context("Could not get Documents dir!")?
            .join("EuroScope")
            .join(new_es_path))
    }
}

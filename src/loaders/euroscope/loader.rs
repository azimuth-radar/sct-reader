use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}, str::FromStr};

use anyhow::Context;
use directories::UserDirs;

use crate::loaders::ese::{self, reader::EseReader, Ese};

use super::{colour::Colour, reader::SctReader, sector::Sector, symbology::{SymbologyAttribute, SymbologyInfo, SymbologyItem}, EsAsr};

#[derive(Debug, Default)]
pub struct EuroScopeResult {
    pub prf_name: String,
    pub prf_file: String,
    pub default_sector_id: String,
    pub sectors: HashMap<String, (Sector, Option<Ese>)>,
    pub symbology: SymbologyInfo,
    pub asrs: HashMap<String, EsAsr>
}

#[derive(Debug)]
pub struct EuroScopeLoader {
    pub prf_file: String,
    pub symbology_file: String,
    pub sector_file: String,
    pub asr_files: Vec<(String, String)>
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
                    match items[0].to_lowercase().as_str() {
                        "settings" => {
                            if items.len() >= 3 {
                                match items[1].to_lowercase().as_str() {
                                    "settingsfilesymbology" => {
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
                        "asrfastkeys" => asrs.push((
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
            asr_files: asrs
        })
    }

    pub fn try_read(&mut self) -> anyhow::Result<EuroScopeResult> {
        let mut ret_val = EuroScopeResult::default();
        ret_val.prf_file = self.prf_file.to_string();
        ret_val.prf_name = Path::new(&self.prf_file).file_stem().unwrap_or_default().to_str().unwrap().to_string();

        // Load symbology
        ret_val.symbology = SymbologyInfo::try_from_file(&self.symbology_file)?;

        // Load Main Sector File
        let sct_reader = SctReader::new(BufReader::new(File::open(&self.sector_file)?));
        let sct_result = sct_reader.try_read()?;
        let ese_file = self.sector_file.replace(".sct", ".ese");
        let sct_ese_result = match std::fs::exists(&ese_file) {
            Ok(true) => {
                if let Ok(file) = File::open(&ese_file) {
                    let reader = EseReader::new(BufReader::new(file));
                    reader.try_read().ok();
                }
                None
            },
            _ => None
        };

        ret_val.default_sector_id = self.sector_file.to_string();
        ret_val.sectors.insert(self.sector_file.to_string(), (sct_result, sct_ese_result));

        // Load ASRs
        for asr_source in &self.asr_files {
            let mut asr = EsAsr::try_from_asr_file(&asr_source.1)?;
            if !asr.1.is_empty() {
                let asr_sector_path = Self::try_convert_es_path(&self.prf_file, &asr.1)?.canonicalize()?.to_str().unwrap().to_owned();
                if !ret_val.sectors.contains_key(&asr_sector_path) {
                    let asr_sct_reader = SctReader::new(BufReader::new(File::open(&asr_sector_path)?));
                    let asr_sct_result = asr_sct_reader.try_read()?;

                    let asr_ese_file = asr_sector_path.replace(".sct", ".ese");
                    let asr_sct_ese_result = match std::fs::exists(&asr_ese_file) {
                        Ok(true) => {
                            if let Ok(file) = File::open(&ese_file) {
                                let reader = EseReader::new(BufReader::new(file));
                                reader.try_read().ok();
                            }
                            None
                        },
                        _ => None
                    };
                    ret_val.sectors.insert(asr_sector_path.to_string(), (asr_sct_result, asr_sct_ese_result));
                }
    
                asr.0.sector_file_id = Some(asr_sector_path.to_string());
            } else {
                asr.0.sector_file_id = Some(ret_val.default_sector_id.clone());
            }
            asr.0.name = Path::new(&asr_source.1).file_stem().unwrap_or_default().to_str().unwrap().to_string();


            ret_val.asrs.insert(asr_source.0.to_string(), asr.0);
        }

        Ok(ret_val)
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

        let path = 
        directories::BaseDirs::new()
        .context("Failed to get User Directories")?
        .config_dir()
        .join("EuroScope")
        .join(new_es_path.clone());


        if path.exists() {
            return Ok(path);
        } 
        

        Ok(UserDirs::new()
            .context("Failed to get User Directories")?
            .document_dir()
            .context("Could not get Documents dir!")?
            .join("EuroScope")
            .join(new_es_path))
    }
}

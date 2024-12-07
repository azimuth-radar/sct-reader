use std::{fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};

use anyhow::Context;
use directories::UserDirs;

use super::{reader::SctReader, sector::Sector};

#[derive(Debug)]
pub struct EuroScopeLoader {
    pub prf_file: String,
    pub symbology_file: String,
    pub sector_file: String,
    pub asrs: Vec<(String, String)>,
    pub sector: Option<Sector>
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
            if line.is_ok() {
                let ln = line?;
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
            asrs,
            sector: None
        })
    }

    fn load_symbology(&mut self) -> anyhow::Result<()> {
        //let file_reader =

        Ok(())
    }

    pub fn process_data(&mut self) -> anyhow::Result<()> {

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

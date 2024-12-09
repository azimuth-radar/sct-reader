use std::{fs::File, path::Path};

use anyhow::Context;
use aviation_calc_util::geo::GeoPoint;
use facility::CrcFacility;
use serde::{Deserialize, Serialize};

pub mod facility;
pub mod eram;
pub mod stars;
pub mod tower;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcVideoMapRef {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub short_name: Option<String>,
    pub source_file_name: String,
    pub stars_brightness_category: String,
    pub stars_id: Option<i32>,
    pub stars_always_visible: bool,
    pub tdm_only: bool
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcTranceiver {
    pub id: String,
    pub name: String,
    pub location: GeoPoint,
    pub height_msl_meters: f64,
    pub height_agl_meters: f64
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrcPackage {
    pub id: String,
    pub video_maps: Vec<CrcVideoMapRef>,
    pub transceivers: Vec<CrcTranceiver>,
    pub visibility_centers: Vec<GeoPoint>,
    pub facility: CrcFacility
}

impl CrcPackage {
    pub fn try_new_from_file(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let package: CrcPackage = serde_json::from_reader::<File, CrcPackage>(File::open(file)?).context("Invalid CRC Json")?;

        Ok(package)
    }
}
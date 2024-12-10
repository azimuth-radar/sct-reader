use std::collections::HashMap;
use std::path::Display;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use crate::loaders::vnas_crc::CrcVideoMapRef;
use crate::loaders::vnas_crc::facility::CrcFacility;
use super::display::AtcDisplay;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtcFacility {
    pub name: String,
    pub displays: Vec<AtcDisplay>,
    pub child_facilities: Vec<AtcFacility>
}

impl AtcFacility {
    pub fn try_from_crc(value: &CrcFacility, maps: &HashMap<String, CrcVideoMapRef>) -> anyhow::Result<Self> {
        let mut children = Vec::new();

        // Process child facilities
        for child in &value.child_facilities {
            children.push(AtcFacility::try_from_crc(child, &maps)?);
        }

        // Process displays
        let mut displays = Vec::new();

        if let Some(stars_cfg) = &value.stars_configuration {
            displays.append(&mut AtcDisplay::from_crc_stars(stars_cfg, &maps));
        }

        if let Some(eram_cfg) = &value.eram_configuration {
            displays.append(&mut AtcDisplay::from_crc_eram(eram_cfg, &maps));
        }

        if let Some(asdex_cfg) = &value.asdex_configuration {
            displays.push(AtcDisplay::from_crc_twr_asdex("asdex".to_string(), "ASDE-X".to_string(), asdex_cfg));
        }
        if let Some(twr_cfg) = &value.tower_cab_configuration {
            displays.push(AtcDisplay::from_crc_twr_asdex("twrcab".to_string(), "Tower Cab".to_string(), twr_cfg));
        }

        Ok(Self {
            name: value.name.to_string(),
            child_facilities: children,
            displays: displays
        })
    }
}
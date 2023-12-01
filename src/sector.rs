use std::collections::HashMap;

use crate::{
    colour::Colour,
    error::Error,
    line::{ColouredLine, LineGroup},
    partial::{
        region::{PartialRegion, PartialRegionGroup},
        sector_info::PartialSectorInfo,
        PartialSector,
    },
    position::{Position, Valid},
    waypoint::{Airport, Fix, Ndb, Vor},
};

#[derive(Debug)]
pub struct Sector {
    pub sector_info: SectorInfo,
    pub colours: HashMap<String, Colour>,
    pub airports: Vec<Airport>,
    pub vors: Vec<Vor>,
    pub ndbs: Vec<Ndb>,
    pub fixes: Vec<Fix>,
    pub artcc_entries: Vec<LineGroup<ColouredLine>>,
    pub artcc_low_entries: Vec<LineGroup<ColouredLine>>,
    pub artcc_high_entries: Vec<LineGroup<ColouredLine>>,
    pub low_airways: Vec<LineGroup<ColouredLine>>,
    pub high_airways: Vec<LineGroup<ColouredLine>>,
    pub sid_entries: Vec<LineGroup<ColouredLine>>,
    pub star_entries: Vec<LineGroup<ColouredLine>>,
    pub geo_entries: Vec<LineGroup<ColouredLine>>,
    pub regions: Vec<RegionGroup>,
    pub labels: Vec<LabelGroup>,

    pub non_critical_errors: Vec<(usize, String, Error)>,
}

impl TryFrom<PartialSector> for Sector {
    type Error = Error;
    fn try_from(value: PartialSector) -> Result<Self, Self::Error> {
        let sector_info = SectorInfo::try_from(value.sector_info)?;
        let regions = value
            .regions
            .into_iter()
            .map(|region_group| RegionGroup::try_from(region_group))
            .collect::<Result<Vec<RegionGroup>, Error>>()?;
        Ok(Sector {
            sector_info,
            colours: value.colours,
            airports: value.airports,
            vors: value.vors,
            ndbs: value.ndbs,
            fixes: value.fixes,
            artcc_entries: value.artcc_entries,
            artcc_low_entries: value.artcc_low_entries,
            artcc_high_entries: value.artcc_high_entries,
            low_airways: value.low_airways,
            high_airways: value.high_airways,
            sid_entries: value.sid_entries,
            star_entries: value.star_entries,
            geo_entries: value.geo_entries,
            regions,
            labels: value.labels,
            non_critical_errors: vec![],
        })
    }
}

#[derive(Debug)]
pub struct RegionGroup {
    pub name: String,
    pub regions: Vec<Region>,
}
impl RegionGroup {
    pub fn new(name: String) -> RegionGroup {
        RegionGroup {
            name,
            regions: vec![],
        }
    }
}

impl TryFrom<PartialRegionGroup> for RegionGroup {
    type Error = Error;
    fn try_from(value: PartialRegionGroup) -> Result<Self, Self::Error> {
        let regions = value
            .regions
            .into_iter()
            .map(Region::try_from)
            .collect::<Result<Vec<_>, Error>>();
        if regions.is_err() {
            println!("NAME: {}", value.name);
        }
        let regions = regions?;
        Ok(RegionGroup {
            name: value.name,
            regions,
        })
    }
}

#[derive(Debug)]
pub struct Region {
    pub colour: Colour,
    pub vertices: Vec<Position<Valid>>,
}
impl TryFrom<PartialRegion> for Region {
    type Error = Error;
    fn try_from(value: PartialRegion) -> Result<Self, Self::Error> {
        Ok(Region {
            colour: value.colour.ok_or_else(|| Error::InvalidRegion)?,
            vertices: value.vertices,
        })
    }
}

#[derive(Debug)]
pub struct LabelGroup {
    pub name: String,
    pub labels: Vec<Label>,
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
    pub position: Position<Valid>,
    pub colour: Colour,
}

#[derive(Debug, Clone)]
pub struct SectorInfo {
    pub name: String,
    pub default_callsign: String,
    pub default_airport: String,
    pub default_centre_pt: Position<Valid>,
    pub n_mi_per_deg_lat: f32,
    pub n_mi_per_deg_lon: f32,
    pub magnetic_variation: f32,
    pub sector_scale: f32,
}

impl TryFrom<PartialSectorInfo> for SectorInfo {
    type Error = Error;
    fn try_from(value: PartialSectorInfo) -> Result<Self, Self::Error> {
        let name = value.name.ok_or(Error::SectorInfoError)?;
        let default_callsign = value.default_callsign.ok_or(Error::SectorInfoError)?;
        let default_airport = value.default_airport.ok_or(Error::SectorInfoError)?;
        let lat = value.default_centre_pt_lat.ok_or(Error::SectorInfoError)?;
        let lon = value.default_centre_pt_lon.ok_or(Error::SectorInfoError)?;
        let default_centre_pt = Position::new(lat, lon).validate()?;
        let n_mi_per_deg_lat = value.n_mi_per_deg_lat.ok_or(Error::SectorInfoError)?;
        let n_mi_per_deg_lon = value.n_mi_per_deg_lon.ok_or(Error::SectorInfoError)?;
        let magnetic_variation = value.magnetic_variation.ok_or(Error::SectorInfoError)?;
        let sector_scale = value.sector_scale.ok_or(Error::SectorInfoError)?;

        Ok(SectorInfo {
            name,
            default_callsign,
            default_airport,
            default_centre_pt,
            n_mi_per_deg_lat,
            n_mi_per_deg_lon,
            magnetic_variation,
            sector_scale,
        })
    }
}

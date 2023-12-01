use std::{collections::HashMap, str::FromStr};

use crate::{
    colour::Colour,
    error::Error,
    line::{ColouredLine, LineGroup},
    position::{self, Heading, Position},
    sector::{Label, LabelGroup},
    waypoint::{self, Airport, Fix, Ndb, RunwayEnd, RunwayModifier, RunwayStrip, Vor},
    AirspaceClass, SectorResult,
};

use self::{
    region::{PartialRegion, PartialRegionGroup},
    sector_info::PartialSectorInfo,
};

pub mod region;
pub mod sector_info;

#[derive(Debug, Default)]
pub struct PartialSector {
    pub colours: HashMap<String, Colour>,
    pub sector_info: PartialSectorInfo,
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
    pub regions: Vec<PartialRegionGroup>,
    pub labels: Vec<LabelGroup>,

    current_region_name: String,
}

impl PartialSector {
    pub fn new() -> PartialSector {
        PartialSector {
            current_region_name: String::from("noname"),
            labels: vec![LabelGroup { name: String::from("SCT2"), labels: vec![] }],
            ..Default::default()
        }
    }

    fn try_fetch_or_decode_colour(&self, value: &str) -> Option<Colour> {
        if let Ok(colour) = Colour::from_str(value) {
            return Some(colour);
        };
        self.colours.get(&value.to_lowercase()).map(|x| *x)
    }
    fn try_fetch_or_decode_lat_lon(&self, lat: &str, lon: &str) -> Option<Position> {
        if let Ok(position) = Position::try_new_from_es(lat, lon) {
            return Some(position.into());
        }

        for fix in &self.fixes {
            if fix.identifier == lat {
                return Some((fix.position.into()));
            }
        }
        for vor in &self.vors {
            if vor.identifier == lat {
                return Some((vor.position.into()));
            }
        }
        for ndb in &self.ndbs {
            if ndb.identifier == lat {
                return Some((ndb.position.into()));
            }
        }
        for airport in &self.airports {
            if airport.identifier == lat {
                return Some((airport.position.into()));
            }
        }

        return None;
    }

    pub fn parse_colour_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let colour_name = sections
            .nth(1)
            .ok_or(Error::InvalidColourDefinition)?
            .to_lowercase();
        let colour_def = sections.next().ok_or(Error::InvalidColourDefinition)?;
        let colour = colour_def.parse::<Colour>()?;
        self.colours.insert(colour_name, colour);
        Ok(())
    }
    pub fn parse_sector_info_line(&mut self, value: &str) -> SectorResult<()> {
        self.sector_info.parse_line(value)
    }
    pub fn parse_airport_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier = sections.next().ok_or(Error::InvalidWaypoint)?.to_owned();
        let tower_frequency = sections.next().ok_or(Error::InvalidWaypoint)?.to_owned();
        let lat = sections.next().ok_or(Error::InvalidWaypoint)?;
        let lon = sections.next().ok_or(Error::InvalidWaypoint)?;
        let position = Position::try_new_from_es(lat, lon)?.validate()?;
        let airspace_class: AirspaceClass =
            sections.next().ok_or(Error::InvalidWaypoint)?.parse()?;

        let airport = Airport {
            identifier,
            position,
            tower_frequency,
            airspace_class,
            runways: vec![],
        };

        self.airports.push(airport);

        Ok(())
    }

    pub fn parse_runway_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier_a = sections.next().ok_or(Error::InvalidRunway)?;
        let identifier_b = sections.next().ok_or(Error::InvalidRunway)?;
        let (number_a, modifier_a) = parse_runway_identifier(identifier_a)?;
        let (number_b, modifier_b) = parse_runway_identifier(identifier_b)?;

        let heading_a = Heading::new(
            sections
                .next()
                .ok_or(Error::InvalidRunway)?
                .parse::<f32>()
                .map_err(|_| Error::InvalidRunway)?,
        )?;
        let heading_b = Heading::new(
            sections
                .next()
                .ok_or(Error::InvalidRunway)?
                .parse::<f32>()
                .map_err(|_| Error::InvalidRunway)?,
        )?;

        let lat_a = sections.next().ok_or(Error::InvalidRunway)?;
        let lon_a = sections.next().ok_or(Error::InvalidRunway)?;

        let lat_b = sections.next().ok_or(Error::InvalidRunway)?;
        let lon_b = sections.next().ok_or(Error::InvalidRunway)?;

        let pos_a = Position::try_new_from_es(lat_a, lon_a)?.validate()?;
        let pos_b = Position::try_new_from_es(lat_b, lon_b)?.validate()?;

        let airport = sections.next().ok_or(Error::InvalidRunway)?;
        let airport = self
            .airports
            .iter_mut()
            .find(|entry| entry.identifier == airport)
            .ok_or(Error::InvalidRunway)?;

        let mut runway_end_a = RunwayEnd {
            number: number_a,
            td_threshold_pos: pos_a,
            se_threshold_pos: pos_b,
            modifier: modifier_a,
            magnetic_hdg: heading_a,
        };

        let mut runway_end_b = RunwayEnd {
            number: number_b,
            td_threshold_pos: pos_b,
            se_threshold_pos: pos_a,
            modifier: modifier_b,
            magnetic_hdg: heading_b,
        };

        if number_a > number_b {
            std::mem::swap(&mut runway_end_a, &mut runway_end_b);
        }

        let runway_strip = RunwayStrip {
            end_a: runway_end_a,
            end_b: runway_end_b,
        };

        airport.runways.push(runway_strip);
        Ok(())
    }

    pub fn parse_vor_or_ndb_line(
        &mut self,
        value: &str,
        beacon_type: BeaconType,
    ) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier = sections.next().ok_or(Error::InvalidVorOrNdb)?.to_owned();
        let frequency = sections.next().ok_or(Error::InvalidVorOrNdb)?.to_owned();
        let lat = sections.next().ok_or(Error::InvalidVorOrNdb)?;
        let lon = sections.next().ok_or(Error::InvalidVorOrNdb)?;
        let position = Position::try_new_from_es(lat, lon)?.validate()?;

        match beacon_type {
            BeaconType::Ndb => {
                let ndb = Ndb {
                    identifier,
                    position,
                    frequency,
                };
                self.ndbs.push(ndb);
            }
            BeaconType::Vor => {
                let vor = Vor {
                    identifier,
                    position,
                    frequency,
                };
                self.vors.push(vor);
            }
        }
        Ok(())
    }

    pub fn parse_fixes_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier = sections.next().ok_or(Error::InvalidFix)?.to_owned();
        let lat = sections.next().ok_or(Error::InvalidFix)?;
        let lon = sections.next().ok_or(Error::InvalidFix)?;
        let position = Position::try_new_from_es(lat, lon)?.validate()?;
        let fix = Fix {
            identifier,
            position,
        };
        self.fixes.push(fix);
        Ok(())
    }

    pub fn parse_artcc_or_airway_line(
        &mut self,
        value: &str,
        line_type: ArtccOrAirwayLineType,
    ) -> SectorResult<()> {
        let mut sections = value.split_whitespace().collect::<Vec<_>>();

        // Get the colour from the last section. If there is one, remove that element.
        let colour = sections
            .last()
            .and_then(|section| self.try_fetch_or_decode_colour(section));
        if colour.is_some() {
            sections.pop();
        };
        //sections: ["AoR", "Milano", "ACC", "N043.34.13.000", "E008.19.18.199", "N043.42.07.000", "E007.50.15.000", "COLOR_AoRcenter1"]

        // Determine whether this is a new section (with a name), or a continuation of a previous section.
        let mut first_coord_index = 0;
        let name = if sections.len() > 4 {
            first_coord_index = sections.len() - 4;
            Some(sections[0..first_coord_index].join(" "))
        } else if sections.len() == 4 {
            None
        } else {
            return Err(Error::InvalidArtccEntry);
        };

        let pos_a = self
            .try_fetch_or_decode_lat_lon(
                sections[first_coord_index],
                sections[first_coord_index + 1],
            )
            .ok_or(Error::InvalidArtccEntry)?;
        let pos_b = self
            .try_fetch_or_decode_lat_lon(
                sections[first_coord_index + 2],
                sections[first_coord_index + 3],
            )
            .ok_or(Error::InvalidArtccEntry)?;

        // Determine which storage to use.
        let storage = match line_type {
            ArtccOrAirwayLineType::Artcc => &mut self.artcc_entries,
            ArtccOrAirwayLineType::ArtccLow => &mut self.artcc_low_entries,
            ArtccOrAirwayLineType::ArtccHigh => &mut self.artcc_high_entries,
            ArtccOrAirwayLineType::LowAirway => &mut self.low_airways,
            ArtccOrAirwayLineType::HighAirway => &mut self.high_airways,
        };

        let name_exists = name.is_some();

        let element = if let Some(name) = name {
            if let Some(element) = storage.iter_mut().find(|element| element.name == name) {
                element
            } else {
                storage.push(LineGroup::new(name, Vec::new()));
                storage.last_mut().unwrap()
            }
        } else {
            storage.last_mut().ok_or(Error::InvalidArtccEntry)?
        };

        let line = pos_a
            .validate()
            .and_then(|pos_a| {
                pos_b
                    .validate()
                    .map(|pos_b| ColouredLine::new(pos_a, pos_b, colour))
            })
            .ok();

        if let Some(line) = line {
            element.lines.push(line);
        } else {
            if !name_exists {
                return Err(Error::InvalidArtccEntry);
            }
        }
        Ok(())
    }

    pub fn parse_sid_star_line(
        &mut self,
        value: &str,
        sid_star_type: SidStarType,
    ) -> SectorResult<()> {
        let name = value.get(0..26).ok_or(Error::InvalidSidStarEntry)?.trim();
        let mut sections = value
            .get(26..)
            .ok_or(Error::InvalidSidStarEntry)?
            .trim()
            .split_whitespace();
        let lat_a = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let lon_a = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let lat_b = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let lon_b = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let colour = sections
            .next()
            .and_then(|x| self.try_fetch_or_decode_colour(x));
        let line = self
            .try_fetch_or_decode_lat_lon(lat_a, lon_a)
            .and_then(|pos| pos.validate().ok())
            .and_then(|start_pos| {
                self.try_fetch_or_decode_lat_lon(lat_b, lon_b)
                    .and_then(|pos| pos.validate().ok())
                    .and_then(|end_pos| Some(ColouredLine::new(start_pos, end_pos, colour)))
            });

        let vec = match sid_star_type {
            SidStarType::Sid => &mut self.sid_entries,
            SidStarType::Star => &mut self.star_entries,
        };

        if name.is_empty() {
            let in_progress_entry = vec.last_mut().ok_or(Error::InvalidSidStarEntry)?;
            if let Some(line) = line {
                in_progress_entry.lines.push(line);
            }
        } else {
            let new_entry = LineGroup::new(
                name.to_owned(),
                if let Some(line) = line {
                    vec![line]
                } else {
                    vec![]
                },
            );
            vec.push(new_entry);
        }
        Ok(())
    }

    pub fn parse_geo_line(&mut self, value: &str) -> SectorResult<()> {
        // Split into sections
        let mut sections = value.split_whitespace().collect::<Vec<_>>();

        // Get the colour, if there is one
        let colour = sections
            .last()
            .and_then(|section| self.try_fetch_or_decode_colour(section));

        // And pop the colour off the end
        if colour.is_some() {
            sections.pop();
        };

        // Get the name if there is one
        // Also determine the index of the first coord
        let mut first_coord_index = 0;
        let name = if sections.len() > 4 {
            first_coord_index = sections.len() - 4;
            Some(sections[0..first_coord_index].join(" "))
        } else if sections.len() == 4 {
            None
        } else {
            return Err(Error::InvalidArtccEntry);
        };

        // Deserialise the positions, but we're not checking to see if they are valid lat / longs yet - only that they're formatted correctly
        let pos_a = self
            .try_fetch_or_decode_lat_lon(
                sections[first_coord_index],
                sections[first_coord_index + 1],
            )
            .ok_or(Error::InvalidGeoEntry)?;
        let pos_b = self
            .try_fetch_or_decode_lat_lon(
                sections[first_coord_index + 2],
                sections[first_coord_index + 3],
            )
            .ok_or(Error::InvalidGeoEntry)?;

        let storage = &mut self.geo_entries;
        let name_exists = name.is_some();

        let element = if let Some(name) = name {
            if let Some(element) = storage.iter_mut().find(|element| element.name == name) {
                element
            } else {
                storage.push(LineGroup::new(name, Vec::new()));
                storage.last_mut().unwrap()
            }
        } else {
            if let Some(entry) = storage.last_mut() {
                entry
            } else {
                storage.push(LineGroup::new("DEFAULT".to_owned(), vec![]));
                storage.last_mut().unwrap()
            }
        };

        let line = pos_a
            .validate()
            .and_then(|pos_a| {
                pos_b
                    .validate()
                    .map(|pos_b| ColouredLine::new(pos_a, pos_b, colour))
            })
            .ok();

        if let Some(line) = line {
            element.lines.push(line);
        } else {
            if !name_exists {
                return Err(Error::InvalidGeoEntry);
            }
        }
        Ok(())
    }

    pub fn parse_region_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace().collect::<Vec<_>>();
        if sections.len() < 2 {
            return Err(Error::InvalidRegion);
        }

        // If a new name is defined here, we'll go ahead and set it for later
        if sections[0] == "REGIONNAME" {
            // We set the current region name
            let name = sections[1..].join(" ");
            self.current_region_name = name.clone();
            return Ok(());
        }
        // If a colour is defined, this is a new region. We see if any with the same name already exist, otherwise create it.
        else if sections.len() == 3 {
            let colour = self
                .try_fetch_or_decode_colour(sections[0])
                .ok_or(Error::InvalidRegion)?;
            if let Some(region_group) = self
                .regions
                .iter_mut()
                .find(|region_group| region_group.name == self.current_region_name)
            {
                region_group.regions.push(PartialRegion {
                    colour: Some(colour),
                    vertices: vec![],
                });
            } else {
                self.regions.push(PartialRegionGroup {
                    name: self.current_region_name.clone(),
                    regions: vec![],
                });
            }
        }

        // Finally we try to get some valid coords
        if let Some(position) = self
            .try_fetch_or_decode_lat_lon(sections[sections.len() - 2], sections[sections.len() - 1])
            .map(|pos| pos.validate().ok())
            .flatten()
        {
            self.regions
                .iter_mut()
                .find(|region_group| region_group.name == self.current_region_name)
                .ok_or(Error::InvalidRegion)?
                .regions
                .last_mut()
                .ok_or(Error::InvalidRegion)?
                .vertices
                .push(position);
        }

        return Ok(());
    }

    pub fn parse_label_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace().collect::<Vec<_>>();
        if sections.len() < 4 {
            return Err(Error::InvalidLabel);
        }
        let colour = self
            .try_fetch_or_decode_colour(sections[sections.len() - 1])
            .ok_or(Error::InvalidLabel)?;
        let position =
            Position::try_new_from_es(sections[sections.len() - 3], sections[sections.len() - 2])
                .and_then(|position| position.validate())?;
        let name = sections[0..sections.len() - 3].join(" ");
        let name = name.trim_matches('"');
        let label = Label {
            name: name.to_owned(),
            position,
            colour,
        };
        self.labels.last_mut().unwrap().labels.push(label);
        Ok(())
    }
}

fn parse_runway_identifier(value: &str) -> SectorResult<(u8, RunwayModifier)> {
    let modifier = if value.ends_with('L') {
        RunwayModifier::Left
    } else if value.ends_with('C') {
        RunwayModifier::Centre
    } else if value.ends_with('R') {
        RunwayModifier::Right
    } else if value.ends_with('G') {
        RunwayModifier::Grass
    } else {
        RunwayModifier::None
    };
    let letters_to_trim: &[_] = &['L', 'R', 'C', 'G'];
    let value = value.trim_end_matches(letters_to_trim);

    let mut number: u8 = value.parse().map_err(|_| Error::InvalidRunway)?;
    if number > 36 {
        return Err(Error::InvalidRunway);
    }
    if number == 0 {
        number = 36;
    }

    Ok((number, modifier))
}

pub enum BeaconType {
    Vor,
    Ndb,
}

pub enum ArtccOrAirwayLineType {
    Artcc,
    ArtccHigh,
    ArtccLow,
    LowAirway,
    HighAirway,
}

pub enum SidStarType {
    Sid,
    Star,
}

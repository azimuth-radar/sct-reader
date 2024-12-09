use std::{collections::HashMap, str::FromStr};

use crate::loaders::euroscope::{colour::Colour, error::Error, partial::PositionCreator, SectorResult};

use super::{Airport, AtcPosition, FreeText, FreeTextGroup, Procedure, ProcedureType, RunwayIdentifier};

#[derive(Default)]
pub struct PartialEse {
    pub colours: HashMap<String, Colour>,
    position_creator: PositionCreator,
    pub free_text: Vec<FreeTextGroup>,
    pub sids_stars: Vec<Airport>,
    pub atc_positions: Vec<AtcPosition>,
}
impl PartialEse {

    pub fn parse_offset(&mut self, value: &str) -> SectorResult<()> {
        let sections = value.split_whitespace().collect::<Vec<_>>();
        if sections.len() == 3 {
            let y_offset: f64 = sections[1].parse().map_err(|_| Error::InvalidOffset)?;
            let x_offset: f64 = sections[2].parse().map_err(|_| Error::InvalidOffset)?;
            self.position_creator.set_offset(x_offset, y_offset);
            return Ok(());
        }
        else if sections.len() == 5 {
            let pos_1 = self.position_creator.try_new_from_es(sections[1], sections[2])?;
            let pos_2 = self.position_creator.try_new_from_es(sections[3], sections[4])?;
            let x_offset = pos_2.lon - pos_1.lon;
            let y_offset = pos_2.lat - pos_1.lat;
            self.position_creator.set_offset(x_offset, y_offset);
            return Ok(());
        }

        return Err(Error::InvalidOffset);
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

    pub fn parse_freetext_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split(':');
        let lat = sections.next().ok_or(Error::InvalidFreetext)?;
        let lon = sections.next().ok_or(Error::InvalidFreetext)?;
        let pos = self.position_creator.try_new_from_es(lat, lon)?.validate()?;
        let mut group_name = sections.next().ok_or(Error::InvalidFreetext)?;
        if group_name.is_empty() {
            group_name = "Default";
        }
        let text = sections.next().ok_or(Error::InvalidFreetext)?;

        let group = match self.free_text.iter_mut().find(|group| group.name == group_name) {
            Some(group) => group,
            None => {
                self.free_text.push(FreeTextGroup { name: group_name.to_owned(), entries: Vec::new() });
                self.free_text.last_mut().unwrap()
            },
        };
        
        group.entries.push(FreeText { position: pos, text: text.to_owned() });

        Ok(())
    }

    pub fn parse_sids_stars_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split(':');
        let proc_type = match sections.next() {
            Some("SID") => ProcedureType::SID,
            Some("STAR") => ProcedureType::STAR,
            _ => return Err(Error::InvalidSidStarEntry),
        };
        let icao_identifier = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        if icao_identifier.len() < 2 {
            return Err(Error::InvalidSidStarEntry);
        }
        let runway_identifier = sections.next().and_then(|rwy| RunwayIdentifier::from_str(rwy).ok()).ok_or(Error::InvalidRunway)?;
        let procedure_identifier = sections.next().ok_or(Error::InvalidSidStarEntry)?.to_owned();
        let route = sections.map(|wp| wp.to_owned()).collect::<Vec<_>>();

        if route.is_empty() {
            return Err(Error::InvalidSidStarEntry);
        }

        // Find airport or create if it doesn't exist
        let airport = match self.sids_stars.iter_mut().find(|airport| airport.identifier == icao_identifier) {
            Some(airport) => airport,
            None => {
                self.sids_stars.push(Airport { identifier: icao_identifier.to_owned(), runways: HashMap::new() });
                self.sids_stars.last_mut().unwrap()
            },
        };

        // Find runway or create if it doesn't exist
        let runway = match airport.runways.get_mut(&runway_identifier) {
            Some(runway) => runway,
            None => {
                airport.runways.insert(runway_identifier.clone(), Vec::new());
                airport.runways.get_mut(&runway_identifier).unwrap()
            }
        };
        let procedure = Procedure {
            identifier: procedure_identifier,
            proc_type,
            route
        };
        runway.push(procedure);
        Ok(())
    }

    pub fn parse_atc_position_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split(':');
        let name = sections.next().ok_or(Error::InvalidAtcPosition)?;
        let rt_callsign = sections.next().ok_or(Error::InvalidAtcPosition)?;
        let radio_freq = sections.next().ok_or(Error::InvalidAtcPosition)?;
        let short_identifier = sections.next().ok_or(Error::InvalidAtcPosition)?;
        let middle = sections.next().ok_or(Error::InvalidAtcPosition)?;
        let left = sections.next().ok_or(Error::InvalidAtcPosition)?;
        let right = sections.next().ok_or(Error::InvalidAtcPosition)?;
        
        if name.is_empty() || rt_callsign.is_empty() || radio_freq.is_empty() || !radio_freq.contains('.') || short_identifier.is_empty() || middle.is_empty() {
            return Err(Error::InvalidAtcPosition);
        }

        let mut long_identifier = 
        if !left.is_empty() {
            format!("{left}_{middle}")
        } else {
            middle.to_string()
        };
        if !right.is_empty() {
            long_identifier.push_str(&format!("_{right}"));
        }
        let start_squawk = sections.nth(2).and_then(|s|s.parse::<u16>().ok());
        let end_squawk = sections.next().and_then(|s|s.parse::<u16>().ok());
        
        let mut vis_centres = [None; 4];
        for i in 0..4 {
            let lat = match sections.next() {
                Some(lat) => lat,
                None => break,
            };
            let lon = match sections.next() {
                Some(lon) => lon,
                None => break,
            };
            let vis_centre = match self.position_creator.try_new_from_es(lat, lon).and_then(|vc| vc.validate()) {
                Ok(vc) => vc,
                Err(_) => break,
            };

            vis_centres[i] = Some(vis_centre);
        }
        let atc_position = AtcPosition {
            name: name.to_owned(),
            rt_callsign: rt_callsign.to_owned(),
            radio_freq: radio_freq.to_owned(),
            short_identifier: short_identifier.to_owned(),
            full_identifier: long_identifier,
            start_squawk,
            end_squawk,
            vis_centres
        };

        self.atc_positions.push(atc_position);

        Ok(())
    }
}
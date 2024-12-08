use std::collections::HashMap;

use crate::loaders::euroscope::{colour::Colour, error::Error, partial::PositionCreator, SectorResult};

use super::{FreeText, FreeTextGroup};

#[derive(Default)]
pub struct PartialEse {
    pub colours: HashMap<String, Colour>,
    position_creator: PositionCreator,
    pub free_text: Vec<FreeTextGroup>,
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
}
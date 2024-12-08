use std::str::FromStr;
use std::io::BufRead;
use crate::loaders::euroscope::error::Error;
use crate::loaders::euroscope::SectorResult;

use super::partial::PartialEse;
use super::Ese;


#[derive(Debug)]
enum FileSection {
    FreeText,
    SidsStars,
    Positions,
    Airspace,
    Radar,
    Ground,
}

impl FromStr for FileSection {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let new_section = match s.to_uppercase().as_str() {
            "[FREETEXT]" => Self::FreeText,
            "[SIDSSTARS]" => Self::SidsStars,
            "[POSITIONS]" => Self::Positions,
            "[AIRSPACE]" => Self::Airspace,
            "[RADAR]" => Self::Radar,
            "[GROUND]" => Self::Ground,
            _ => return Err(Error::InvalidFileSection),
        };
        Ok(new_section)
    }
}

pub struct EseReader<R: BufRead> {
    source: R,
    current_section: FileSection,
    partial_ese: PartialEse,
    errors: Vec<(usize, String, Error)>,
}

impl<R: BufRead> EseReader<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            current_section: FileSection::FreeText,
            partial_ese: PartialEse::default(),
            errors: vec![],
        }
    }

    pub fn try_read(mut self) -> SectorResult<Ese> {
        for (mut line_number, line) in self.source.lines().enumerate() {
            if let Ok(line) = line {
                let mut line = line.trim_end();
                line_number += 1;

                if line.is_empty() || line.starts_with(';') {
                    continue;
                }
                if line.contains(';') {
                    let mut line_split = line.split(';');
                    line = line_split.next().unwrap().trim_end();
                }
                if line.starts_with('[') {
                    match FileSection::from_str(line) {
                        Ok(new_section) => self.current_section = new_section,
                        Err(e) => self.errors.push((line_number + 1, line.to_owned(), e)),
                    }
                    continue;
                }
                if line.starts_with("OFFSET") {
                    if let Err(e) = self.partial_ese.parse_offset(line) {
                        self.errors.push((line_number, line.to_owned(), e));
                    }
                    continue;
                }
                if line.starts_with("#define") {
                    if let Err(e) = self.partial_ese.parse_colour_line(line) {
                        self.errors.push((line_number, line.to_owned(), e));
                    }
                    continue;
                }

                let result = match self.current_section {
                    FileSection::FreeText => self.partial_ese.parse_freetext_line(line),
                    _ => continue,
                    // FileSection::SidsStars => todo!(),
                    // FileSection::Positions => todo!(),
                    // FileSection::Airspace => todo!(),
                    // FileSection::Radar => todo!(),
                    // FileSection::Ground => todo!(),
                };
                if let Err(e) = result {
                    self.errors.push((line_number, line.to_owned(), e));
                }
            }
        }

        let mut ese: Ese = self.partial_ese.try_into()?;
        ese.non_critical_errors = self.errors;
        Ok(ese)
    }
}
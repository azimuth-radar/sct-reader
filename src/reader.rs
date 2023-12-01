use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    time::Instant,
};

use crate::{
    colour::Colour,
    error::Error,
    partial::{ArtccOrAirwayLineType, BeaconType, PartialSector, SidStarType},
    sector::Sector,
    SectorResult,
};
use std::io::Write;

pub struct SctReader<R: BufRead> {
    source: R,
    current_section: FileSection,
    partial_sector: PartialSector,
    errors: Vec<(usize, String, Error)>,
}
impl<R: BufRead> SctReader<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            current_section: FileSection::ColourDefinitions,
            partial_sector: PartialSector::new(),
            errors: vec![],
        }
    }

    pub fn try_read(mut self) -> SectorResult<Sector> {
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
                    match parse_file_section(line) {
                        Ok(new_section) => self.current_section = new_section,
                        Err(e) => self.errors.push((line_number + 1, line.to_owned(), e)),
                    }
                    continue;
                }
                if line.starts_with("#define") {
                    self.current_section = FileSection::ColourDefinitions;
                }

                let result = match self.current_section {
                    FileSection::ColourDefinitions => self.partial_sector.parse_colour_line(line),
                    FileSection::Info => self.partial_sector.parse_sector_info_line(line),
                    FileSection::Airport => self.partial_sector.parse_airport_line(line),
                    FileSection::Runway => self.partial_sector.parse_runway_line(line),
                    FileSection::Vor => self
                        .partial_sector
                        .parse_vor_or_ndb_line(line, BeaconType::Vor),
                    FileSection::Ndb => self
                        .partial_sector
                        .parse_vor_or_ndb_line(line, BeaconType::Ndb),
                    FileSection::Fixes => self.partial_sector.parse_fixes_line(line),
                    FileSection::Artcc => self
                        .partial_sector
                        .parse_artcc_or_airway_line(line, ArtccOrAirwayLineType::Artcc),
                    FileSection::ArtccHigh => self
                        .partial_sector
                        .parse_artcc_or_airway_line(line, ArtccOrAirwayLineType::ArtccHigh),
                    FileSection::ArtccLow => self
                        .partial_sector
                        .parse_artcc_or_airway_line(line, ArtccOrAirwayLineType::ArtccLow),
                    FileSection::LowAirway => self
                        .partial_sector
                        .parse_artcc_or_airway_line(line, ArtccOrAirwayLineType::LowAirway),
                    FileSection::HighAirway => self
                        .partial_sector
                        .parse_artcc_or_airway_line(line, ArtccOrAirwayLineType::HighAirway),
                    FileSection::Sid => self
                        .partial_sector
                        .parse_sid_star_line(line, SidStarType::Sid),
                    FileSection::Star => self
                        .partial_sector
                        .parse_sid_star_line(line, SidStarType::Star),
                    FileSection::Geo => self.partial_sector.parse_geo_line(line),
                    FileSection::Regions => self.partial_sector.parse_region_line(line),
                    FileSection::Labels => self.partial_sector.parse_label_line(line),
                };
                if let Err(e) = result {
                    self.errors.push((line_number + 1, line.to_owned(), e));
                }
            }
        }

        let mut sector: Sector = self.partial_sector.try_into()?;
        sector.non_critical_errors = self.errors;
        Ok(sector)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileSection {
    ColourDefinitions,
    Info,
    Vor,
    Ndb,
    Airport,
    Runway,
    Fixes,
    Geo,
    LowAirway,
    HighAirway,
    Artcc,
    ArtccHigh,
    ArtccLow,
    Sid,
    Star,
    Regions,
    Labels,
}

fn parse_file_section(value: &str) -> SectorResult<FileSection> {
    let new_section = match value.to_uppercase().as_str() {
        "[INFO]" => FileSection::Info,
        "[AIRPORT]" => FileSection::Airport,
        "[VOR]" => FileSection::Vor,
        "[NDB]" => FileSection::Ndb,
        "[RUNWAY]" => FileSection::Runway,
        "[FIXES]" => FileSection::Fixes,
        "[ARTCC]" => FileSection::Artcc,
        "[ARTCC HIGH]" => FileSection::ArtccHigh,
        "[ARTCC LOW]" => FileSection::ArtccLow,
        "[SID]" => FileSection::Sid,
        "[STAR]" => FileSection::Star,
        "[LOW AIRWAY]" => FileSection::LowAirway,
        "[HIGH AIRWAY]" => FileSection::HighAirway,
        "[GEO]" => FileSection::Geo,
        "[REGIONS]" => FileSection::Regions,
        "[LABELS]" => FileSection::Labels,
        _ => return Err(Error::InvalidFileSection),
    };
    Ok(new_section)
}

#[test]
fn test() {
    let file =
        File::open(r#"C:\Users\chpme\AppData\Roaming\EuroScope\UK\Data\Sector\UK_2023_11.sct"#)
            .unwrap();
    let reader = BufReader::new(file);
    let sct_reader = SctReader::new(reader);
    let timer = Instant::now();
    match sct_reader.try_read() {
        Ok(sector) => {
            let elapsed = timer.elapsed();
            let mut output = BufWriter::new(File::create("output.txt").unwrap());
            writeln!(output, "Took {} ms", elapsed.as_millis()).unwrap();
            write!(output, "{:#?}", sector).unwrap();
            writeln!(output).unwrap();
            for (line_number, line, error) in sector.non_critical_errors {
                writeln!(output, "{} on line {}:", error, line_number).unwrap();
                writeln!(output, "{}", line).unwrap();
                writeln!(output).unwrap();
            }
        }
        Err(error) => println!("{:#?}", error),
    }
}

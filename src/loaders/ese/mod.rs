use std::{collections::HashMap, fmt::Display, fs::File, io::{BufRead, BufReader}, str::FromStr};

use partial::PartialEse;

use super::euroscope::{self, colour::Colour, error::Error, position::{Position, Valid}, waypoint::RunwayModifier};


pub mod reader;
pub(crate) mod partial;







#[derive(Debug)]
pub struct Ese {
    pub colours: HashMap<String, Colour>,
    pub free_text: Vec<FreeTextGroup>,
    pub sids_stars: Vec<Airport>,
    pub non_critical_errors: Vec<(usize, String, Error)>,
    pub atc_positions: Vec<AtcPosition>,
}
impl TryFrom<PartialEse> for Ese {
    type Error = Error;
    fn try_from(value: PartialEse) -> Result<Self, Self::Error> {
        let ese = Ese {
            colours: value.colours,
            free_text: value.free_text,
            sids_stars: value.sids_stars,
            atc_positions: value.atc_positions,
            non_critical_errors: vec![],
        };
        Ok(ese)
    }
}
#[derive(Debug)]
pub struct FreeTextGroup {
    pub name: String,
    pub entries: Vec<FreeText>,
}

#[derive(Debug)]
pub struct FreeText {
    pub position: Position<Valid>,
    pub text: String,
}


#[derive(Debug)]
pub struct Airport {
    pub identifier: String,
    pub runways: HashMap<RunwayIdentifier, Vec<Procedure>>
}

#[derive(Debug)]
pub enum ProcedureType {
    SID,
    STAR,
}

#[derive(Debug)]
pub struct Procedure {
    pub proc_type: ProcedureType,
    pub identifier: String,
    pub route: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RunwayIdentifier {
    number: u8,
    modifier: RunwayModifier
}
impl FromStr for RunwayIdentifier {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (number, modifier) = euroscope::partial::parse_runway_identifier(s)?;
        Ok(RunwayIdentifier {
            number,
            modifier
        })
    }
}
impl RunwayIdentifier {
    pub fn number(&self) -> u8 {
        self.number
    }
    pub fn modifier(&self) -> RunwayModifier {
        self.modifier.clone()
    }
    pub fn number_and_modifier(&self) -> (u8, RunwayModifier) {
        (self.number, self.modifier.clone())
    }
}
impl Display for RunwayIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}{}", self.number, self.modifier)
    }
}

#[derive(Debug)]
pub struct AtcPosition {
    pub name: String,
    pub rt_callsign: String,
    pub radio_freq: String,
    pub short_identifier: String,
    pub full_identifier: String,
    pub start_squawk: Option<u16>,
    pub end_squawk: Option<u16>,
    pub vis_centres: [Option<Position<Valid>>; 4]
}



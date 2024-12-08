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
}
impl TryFrom<PartialEse> for Ese {
    type Error = Error;
    fn try_from(value: PartialEse) -> Result<Self, Self::Error> {
        let ese = Ese {
            colours: value.colours,
            free_text: value.free_text,
            sids_stars: value.sids_stars,
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


#[test]
fn test_ese() {
    let ese_file = BufReader::new(File::open("/home/caspian/Downloads/uk_controller_pack_2024_12/UK/Data/Sector/UK_2024_12.ese").unwrap());
    let ese_reader = reader::EseReader::new(ese_file);
    let ese = ese_reader.try_read().unwrap();
    println!("{:#?}", ese.sids_stars);
    println!("{:#?}", ese.non_critical_errors);
}
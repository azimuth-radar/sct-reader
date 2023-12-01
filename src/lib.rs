#![allow(unused)]

use std::str::FromStr;

use error::Error;
pub mod colour;
pub mod error;
pub mod line;
mod partial;
pub mod position;
pub mod reader;
pub mod sector;
pub mod waypoint;

pub type SectorResult<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AirspaceClass {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
impl FromStr for AirspaceClass {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "A" => AirspaceClass::A,
            "B" => AirspaceClass::B,
            "C" => AirspaceClass::C,
            "D" => AirspaceClass::D,
            "E" => AirspaceClass::E,
            "F" => AirspaceClass::F,
            "G" => AirspaceClass::G,
            _ => return Err(Error::InvalidAirspaceClass),
        };
        Ok(result)
    }
}

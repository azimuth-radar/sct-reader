use std::fmt::Display;

use crate::{
    position::{Heading, Position, Valid},
    AirspaceClass,
};

pub trait Waypoint {
    fn identifier(&self) -> &String;
    fn position(&self) -> Position<Valid>;
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub identifier: String,
    pub position: Position<Valid>,
}
impl Waypoint for Fix {
    fn identifier(&self) -> &String {
        &self.identifier
    }
    fn position(&self) -> Position<Valid> {
        self.position
    }
}

#[derive(Debug, Clone)]
pub struct Vor {
    pub identifier: String,
    pub position: Position<Valid>,
    pub frequency: String,
}
impl Waypoint for Vor {
    fn identifier(&self) -> &String {
        &self.identifier
    }
    fn position(&self) -> Position<Valid> {
        self.position
    }
}
impl Vor {
    pub fn frequency(&self) -> &String {
        &self.frequency
    }
}

#[derive(Debug, Clone)]
pub struct Ndb {
    pub identifier: String,
    pub position: Position<Valid>,
    pub frequency: String,
}
impl Waypoint for Ndb {
    fn identifier(&self) -> &String {
        &self.identifier
    }
    fn position(&self) -> Position<Valid> {
        self.position
    }
}
impl Ndb {
    pub fn frequency(&self) -> &String {
        &self.frequency
    }
}

#[derive(Debug, Clone)]
pub struct Airport {
    pub identifier: String,
    pub position: Position<Valid>,
    pub tower_frequency: String,
    pub airspace_class: AirspaceClass,
    pub runways: Vec<RunwayStrip>,
}
impl Waypoint for Airport {
    fn identifier(&self) -> &String {
        &self.identifier
    }
    fn position(&self) -> Position<Valid> {
        self.position
    }
}
impl Airport {
    pub fn tower_frequency(&self) -> &String {
        &self.tower_frequency
    }
    pub fn airspace_class(&self) -> AirspaceClass {
        self.airspace_class
    }
}

#[derive(Debug, Clone)]
pub struct RunwayStrip {
    pub end_a: RunwayEnd,
    pub end_b: RunwayEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunwayEnd {
    pub number: u8,
    pub td_threshold_pos: Position<Valid>,
    pub se_threshold_pos: Position<Valid>,
    pub modifier: RunwayModifier,
    pub magnetic_hdg: Heading,
}
impl RunwayEnd {
    pub fn identifier(&self) -> String {
        format!("{:02}{}", self.number, self.modifier)
    }
    pub fn reciprocal(&self) -> RunwayEnd {
        let number = self.reciprocal_number();
        let td_threshold_pos = self.se_threshold_pos;
        let se_threshold_pos = self.td_threshold_pos;
        let modifier = self.modifier.reciprocal();
        let magnetic_hdg = self.magnetic_hdg.reciprocal();
        RunwayEnd {
            number,
            td_threshold_pos,
            se_threshold_pos,
            modifier,
            magnetic_hdg,
        }
    }
    fn reciprocal_number(&self) -> u8 {
        if self.number > 18 {
            self.number - 18
        } else {
            self.number + 18
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunwayModifier {
    Left,
    Right,
    Centre,
    Grass,
    None,
}
impl RunwayModifier {
    pub fn reciprocal(&self) -> RunwayModifier {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Centre => Self::Centre,
            Self::Grass => Self::Grass,
            Self::None => Self::None,
        }
    }
}

impl Display for RunwayModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Left => "L",
                Self::Right => "R",
                Self::Centre => "C",
                Self::Grass => "G",
                Self::None => "",
            }
        )
    }
}

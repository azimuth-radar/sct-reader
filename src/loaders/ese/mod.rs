use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, str::FromStr};

use partial::PartialEse;

use super::euroscope::{colour::Colour, error::Error, position::{Position, Valid}};


pub mod reader;
pub(crate) mod partial;







#[derive(Debug)]
pub struct Ese {
    pub colours: HashMap<String, Colour>,
    pub free_text: Vec<FreeTextGroup>,
    pub non_critical_errors: Vec<(usize, String, Error)>,
}
impl TryFrom<PartialEse> for Ese {
    type Error = Error;
    fn try_from(value: PartialEse) -> Result<Self, Self::Error> {
        let ese = Ese {
            colours: value.colours,
            free_text: value.free_text,
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
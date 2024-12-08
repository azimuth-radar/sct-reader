use crate::loaders::euroscope::{error::Error, position, SectorResult};

use super::PositionCreator;


#[derive(Debug, Clone, Default)]
pub struct PartialSectorInfo {
    pub name: Option<String>,
    pub default_callsign: Option<String>,
    pub default_airport: Option<String>,
    pub default_centre_pt_lat: Option<f64>,
    pub default_centre_pt_lon: Option<f64>,
    pub n_mi_per_deg_lat: Option<f32>,
    pub n_mi_per_deg_lon: Option<f32>,
    pub magnetic_variation: Option<f32>,
    pub sector_scale: Option<f32>,

    current_line: usize,
}
impl PartialSectorInfo {
    pub fn parse_line(&mut self, value: &str, position_creator: &PositionCreator) -> SectorResult<()> {
        self.current_line += 1;
        //println!("Line {}: |{value}|", self.current_line);
        match self.current_line {
            1 => self.name = Some(value.to_owned()),
            2 => self.default_callsign = Some(value.to_owned()),
            3 => self.default_airport = Some(value.to_owned()),
            4 => self.default_centre_pt_lat = {
                let (_, y_offset) = position_creator.offset();
                position::coord_from_es(value).map(|lat| lat + y_offset)
            },
            5 => self.default_centre_pt_lon = {
                let (x_offset, _) = position_creator.offset();
                position::coord_from_es(value).map(|lon| lon + x_offset)
            },
            6 => {
                self.n_mi_per_deg_lat =
                    Some(value.parse::<f32>().map_err(|_| Error::SectorInfoError)?)
            }
            7 => {
                self.n_mi_per_deg_lon =
                    Some(value.parse::<f32>().map_err(|_| Error::SectorInfoError)?)
            }
            8 => {
                self.magnetic_variation =
                    Some(value.parse::<f32>().map_err(|_| Error::SectorInfoError)?)
            }
            9 => {
                self.sector_scale = Some(value.parse::<f32>().map_err(|_| Error::SectorInfoError)?)
            }
            _ => return Err(Error::SectorInfoError),
        }

        Ok(())
    }
}

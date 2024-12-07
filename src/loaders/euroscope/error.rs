use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    MissingMetadata,
    IoError,
    InvalidColourDefinition,
    InvalidFileSection,
    InvalidCoordinate,
    SectorInfoError,
    InvalidAirspaceClass,
    InvalidWaypoint,
    InvalidPosition,
    InvalidRunway,
    InvalidHeading,
    InvalidVorOrNdb,
    InvalidFix,
    InvalidArtccEntry,
    InvalidSidStarEntry,
    InvalidGeoEntry,
    InvalidRegion,
    InvalidLabel,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MissingMetadata => "Missing metadata",
                Self::IoError => "Unable to read the source",
                Self::InvalidColourDefinition => "Invalid colour definition",
                Self::InvalidFileSection => "Invalid file section",
                Self::InvalidCoordinate => "Invalid coordinate",
                Self::SectorInfoError => "Sector information error",
                Self::InvalidAirspaceClass => "Invalid airspace class",
                Self::InvalidWaypoint => "Invalid waypoint",
                Self::InvalidPosition => "Invalid position",
                Self::InvalidRunway => "Invalid runway",
                Self::InvalidHeading => "Invalid heading",
                Self::InvalidVorOrNdb => "Invalid VOR or NDB",
                Self::InvalidFix => "Invalid fix",
                Self::InvalidArtccEntry => "Invalid ARTCC entry",
                Self::InvalidSidStarEntry => "Invalid SID / STAR entry",
                Self::InvalidGeoEntry => "Invalid geo Entry",
                Self::InvalidRegion => "Invalid region",
                Self::InvalidLabel => "Invalid label",
            }
        )
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}

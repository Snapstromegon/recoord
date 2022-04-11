#![deny(
    warnings,
    unsafe_code,
    missing_docs,
    clippy::missing_docs_in_private_items
)]

//! Recoord is a create for handling work with coordinates
//!
//! All corrdinates are always converted to the latitude and longitude float format

use std::{
    fmt,
    fmt::{Display, Formatter},
    num::ParseFloatError,
};
mod formats;
#[cfg(feature = "parse_str_dd")]
pub use formats::dd::DDCoordinate;
#[cfg(feature = "parse_str_dms")]
pub use formats::dms::DMSCoordinate;

use thiserror::Error;

/// The base coordinate struct.
/// It stores the location as latitude, longitude floats
#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    /// Longitude of the coordinate (-90 - 90)
    pub lat: f64,
    /// Latitude of the coordinate (-180 - 180)
    pub lng: f64,
}

impl Coordinate {
    /// Create a new coordinate with longitude and latitude
    ///
    /// ```
    /// /// Normal Coordinate creation
    /// # use recoord::Coordinate;
    /// let manual = Coordinate { lat: 10., lng: 20. };
    /// let coordinate = Coordinate::new(10., 20.);
    /// assert_eq!(coordinate, manual)
    /// ```
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{},{}", self.lat, self.lng)
    }
}

/// Error when handling coordinates
#[derive(Debug, Error)]
pub enum CoordinateError {
    /// No parser available - enable them via features
    #[error("No parser available - enable them via features")]
    MissingParser,
    /// Value can't be converted into a coordinate
    #[error("Value can't be converted into a coordinate")]
    InvalidValue,
    /// String passed into from_str was malformed
    #[cfg(feature = "parse_str_dd")]
    #[cfg(feature = "parse_str_dms")]
    #[error("String passed into from_str was malformed")]
    Malformed,
    /// String passed into from_str contained invalid floats
    #[cfg(feature = "parse_str_dd")]
    #[cfg(feature = "parse_str_dms")]
    #[error("String passed into from_str contained invalid floats")]
    ParseFloatError(#[from] ParseFloatError),
    /// Location not resolvable
    #[cfg(feature = "resolve_osm")]
    #[error("Location not resolvable")]
    Unresolveable,
    /// There was a problem connecting to the API
    #[cfg(feature = "resolve_osm")]
    #[error("There was a problem connecting to the API")]
    ReqwestError(#[from] reqwest::Error),
}

impl TryFrom<(f64, f64)> for Coordinate {
    type Error = CoordinateError;
    /// Try to convert a tuple of coordinates into a Coordinate struct
    ///
    /// ```
    /// /// Parsing works
    /// # use recoord::Coordinate;
    /// let from = Coordinate::try_from((10., 20.));
    /// assert_eq!(Ok(Coordinate { lat: 10.0, lng: 20.0}), from);
    /// ```
    ///
    /// ```
    /// /// Detect invalid values
    /// # use recoord::{Coordinate, CoordinateError};
    /// let from = Coordinate::try_from((100., 20.));
    /// assert_eq!(Err(CoordinateError::InvalidValue), from);
    /// ```
    fn try_from(tupl_coord: (f64, f64)) -> Result<Self, Self::Error> {
        match tupl_coord {
            (lat, lng) if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) => {
                Ok(Self { lat, lng })
            }
            _ => Err(CoordinateError::InvalidValue),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     #[cfg(feature = "parse_str_dd")]
//     #[test]
//     fn parse_str_dd_integer() {
//         use crate::Coordinate;

//         let expected = Coordinate { lat: 10., lng: 20. };
//         let real = Coordinate::parse_str_dd("10,20").unwrap();
//         assert_eq!(expected, real);
//     }
//     #[cfg(feature = "parse_str_dd")]
//     #[test]
//     fn parse_str_dd_float() {
//         use crate::Coordinate;

//         let expected = Coordinate { lat: 10., lng: 20. };
//         let real = Coordinate::parse_str_dd("10.0,20.0").unwrap();
//         assert_eq!(expected, real);
//     }
//     #[cfg(feature = "parse_str_dd")]
//     #[test]
//     fn parse_str_dd_invalid() {
//         use crate::{Coordinate, CoordinateError};

//         match Coordinate::parse_str_dd("Asd,20.0") {
//             Err(CoordinateError::Malformed) => {}
//             Err(_) => panic!("Wrong Error"),
//             Ok(_) => panic!("Should've failed"),
//         }
//     }
// }

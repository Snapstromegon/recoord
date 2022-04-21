#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    clippy::missing_docs_in_private_items
)]

//! Recoord is a create for handling work with coordinates
//!
//! All corrdinates are always converted to the latitude and longitude float format

use std::{
    fmt,
    fmt::{Display, Formatter},
};
/// A wrapper around different coordinate formats
pub mod formats;

/// A wrapper around differend resolvers for Coordinates
pub mod resolvers;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "format_any")]
use std::str::FromStr;

#[cfg(any(feature = "format_dd", feature = "format_dms", feature = "resolve_osm"))]
use std::num::ParseFloatError;

use thiserror::Error;

/// The base coordinate struct.
/// It stores the location as latitude, longitude floats
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[cfg(any(
        feature = "format_dd",
        feature = "format_dms",
        feature = "format_geohash"
    ))]
    #[error("String passed into from_str was malformed")]
    Malformed,
    /// String passed into from_str contained invalid floats
    #[cfg(any(feature = "format_dd", feature = "format_dms", feature = "resolve_osm"))]
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
    /// assert_eq!(Coordinate { lat: 10.0, lng: 20.0}, from.unwrap());
    /// ```
    ///
    /// ```
    /// /// Detect invalid values
    /// # use recoord::{Coordinate, CoordinateError};
    /// let from = Coordinate::try_from((100., 20.));
    /// assert!(from.is_err());
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

#[cfg(feature = "format_any")]
impl FromStr for Coordinate {
    type Err = CoordinateError;

    fn from_str(str_coords: &str) -> Result<Self, Self::Err> {
        let mut result: Result<Coordinate, CoordinateError> = Err(CoordinateError::MissingParser);

        #[cfg(feature = "format_dd")]
        {
            result = result
                .or_else(|_| formats::dd::DDCoordinate::from_str(str_coords).map(Coordinate::from));
        }
        #[cfg(feature = "format_dms")]
        {
            result = result.or_else(|_| {
                formats::dms::DMSCoordinate::from_str(str_coords).map(Coordinate::from)
            });
        }
        #[cfg(feature = "format_geohash")]
        {
            result = result
                .or_else(|_| formats::geohash::Geohash::from_str(str_coords).map(Coordinate::from));
        }

        result
    }
}

// /// Resolver for strings to Coordinates - this should be used for more expensive (and async) resolving
// pub trait Resolver {
//     /// Resolve a &str to a Coordinate
//     fn resolve(s: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Coordinate, CoordinateError>> + Send + '_>>;
// }

// #[cfg(test)]
// mod tests {
//     #[cfg(feature = "format_dd")]
//     #[test]
//     fn format_dd_integer() {
//         use crate::Coordinate;

//         let expected = Coordinate { lat: 10., lng: 20. };
//         let real = Coordinate::format_dd("10,20").unwrap();
//         assert_eq!(expected, real);
//     }
//     #[cfg(feature = "format_dd")]
//     #[test]
//     fn format_dd_float() {
//         use crate::Coordinate;

//         let expected = Coordinate { lat: 10., lng: 20. };
//         let real = Coordinate::format_dd("10.0,20.0").unwrap();
//         assert_eq!(expected, real);
//     }
//     #[cfg(feature = "format_dd")]
//     #[test]
//     fn format_dd_invalid() {
//         use crate::{Coordinate, CoordinateError};

//         match Coordinate::format_dd("Asd,20.0") {
//             Err(CoordinateError::Malformed) => {}
//             Err(_) => panic!("Wrong Error"),
//             Ok(_) => panic!("Should've failed"),
//         }
//     }
// }

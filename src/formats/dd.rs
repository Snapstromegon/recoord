use core::fmt;
use std::{fmt::Display, str::FromStr};

use crate::{Coordinate, CoordinateError};
use regex::Regex;

/// A Coordinate in the floating point representation
/// (e.g. 12.345,6.789)
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DDCoordinate {
    /// Latitude of the Coordinate
    lat: f64,
    /// Longitude of the coordinate
    lng: f64,
}

impl FromStr for DDCoordinate {
    type Err = CoordinateError;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let trans_str_coords = str.to_uppercase();
        let trans_str_coords = trans_str_coords.trim();
        let decimal_coords_re =
            Regex::new(r"^(?P<lat>[+-]?\d+(\.\d+)?)\s*[,\./]\s*(?P<lng>[+-]?\d+(\.\d+)?)$")
                .unwrap();
        let re_captures = decimal_coords_re.captures(trans_str_coords);

        if let Some(captures) = re_captures {
            if let (Some(lng), Some(lat)) = (captures.name("lng"), captures.name("lat")) {
                return Ok(DDCoordinate {
                    lng: lng.as_str().parse()?,
                    lat: lat.as_str().parse()?,
                });
            }
        }
        Err(CoordinateError::Malformed)
    }
}

impl Display for DDCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.lat, self.lng)
    }
}

impl From<DDCoordinate> for Coordinate {
    fn from(dd_coord: DDCoordinate) -> Self {
        Self {
            lat: dd_coord.lat,
            lng: dd_coord.lng,
        }
    }
}

impl From<Coordinate> for DDCoordinate {
    fn from(coord: Coordinate) -> Self {
        Self {
            lat: coord.lat,
            lng: coord.lng,
        }
    }
}

#![deny(
    warnings,
    unsafe_code,
    missing_docs,
    clippy::missing_docs_in_private_items
)]

//! Recoord is a create for handling work with coordinates
//!
//! All corrdinates are always converted to the latitude and longitude float format

#[cfg(feature = "parse_str_dd")]
#[cfg(feature = "parse_str_dms")]
use regex::Regex;
#[cfg(feature = "parse_str_dd")]
#[cfg(feature = "parse_str_dms")]
use std::num::ParseFloatError;
use std::{str::FromStr, fmt};

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

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.lat, self.lng)
    }
}

/// Error when handling coordinates
#[derive(Debug, PartialEq)]
pub enum CoordinateError {
    /// No parser available - enable them via features
    MissingParser,
    /// A coordinate has an invalid value
    InvalidValue,
    /// String passed into from_str was malformed
    #[cfg(feature = "parse_str_dd")]
    #[cfg(feature = "parse_str_dms")]
    Malformed,
    /// String passed into from_str contained invalid floats
    #[cfg(feature = "parse_str_dd")]
    #[cfg(feature = "parse_str_dms")]
    ParseFloatError(ParseFloatError),
}

#[cfg(feature = "parse_str_dd")]
#[cfg(feature = "parse_str_dms")]
impl From<ParseFloatError> for CoordinateError {
    fn from(err: ParseFloatError) -> Self {
        Self::ParseFloatError(err)
    }
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

#[cfg(feature = "parse_str_dd")]
impl Coordinate {
    /// Parse a Coordinate from a string in the format "<lat: f64>,<lng: f64>"
    fn parse_str_dd(str_coords: &str) -> Result<Self, CoordinateError> {
        let trans_str_coords = str_coords.to_uppercase();
        let trans_str_coords = trans_str_coords.trim();
        let decimal_coords_re =
            Regex::new(r"^(?P<lat>[+-]?\d+(\.\d+)?)\s*[,\./]\s*(?P<lng>[+-]?\d+(\.\d+)?)$")
                .unwrap();
        let re_captures = decimal_coords_re.captures(trans_str_coords);

        if let Some(captures) = re_captures {
            if let (Some(lng), Some(lat)) = (captures.name("lng"), captures.name("lat")) {
                return Ok(Coordinate {
                    lng: lng.as_str().parse()?,
                    lat: lat.as_str().parse()?,
                });
            }
        }
        Err(CoordinateError::Malformed)
    }
}

/// Possible compass directions
#[derive(PartialEq, Eq)]
#[cfg(feature = "parse_str_dms")]
enum CompassDirection {
    /// North
    North,
    /// East
    East,
    /// South
    South,
    /// West
    West,
}

#[cfg(feature = "parse_str_dms")]
impl From<&str> for CompassDirection {
    fn from(dir: &str) -> Self {
        match &dir.to_uppercase()[..] {
            "N" | "NORTH" => CompassDirection::North,
            "E" | "EAST" => CompassDirection::East,
            "S" | "SOUTH" => CompassDirection::South,
            "W" | "WEST" => CompassDirection::West,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "parse_str_dms")]
impl Coordinate {
    /// Convert a sexagesimal coordinate to a decimal one.
    fn sexagesimal_to_decimal(degree: f64, minutes: Option<f64>, seconds: Option<f64>) -> f64 {
        degree + minutes.unwrap_or(0.) / 60. + seconds.unwrap_or(0.) / 60. / 60.
    }

    /// Parse a Coordinate from a string in the format "<lat: f64>,<lng: f64>"
    fn parse_str_dms(str_coords: &str) -> Result<Self, CoordinateError> {
        let trans_str_coords = str_coords.to_uppercase();
        let trans_str_coords = trans_str_coords.trim();
        let long_lat_re = Regex::new("^(?P<lat_deg>\\d+(\\.\\d+)?)°((?P<lat_min>\\d+(\\.\\d+)?)')?((?P<lat_sec>\\d+(\\.\\d+)?)\"?)(?P<n_s>[NS])\\s*(?P<long_deg>\\d+(\\.\\d+)?)°((?P<long_min>\\d+(\\.\\d+)?)')?((?P<long_sec>\\d+(\\.\\d+)?)\")?(?P<e_w>[EW])$").unwrap();
        let re_captures = long_lat_re.captures(trans_str_coords);
        if let Some(captures) = re_captures {
            if let (
                Some(lat_deg),
                lat_min,
                lat_sec,
                Some(n_s),
                Some(long_deg),
                long_min,
                long_sec,
                Some(e_w),
            ) = (
                captures.name("lat_deg"),
                captures.name("lat_min"),
                captures.name("lat_sec"),
                captures.name("n_s"),
                captures.name("long_deg"),
                captures.name("long_min"),
                captures.name("long_sec"),
                captures.name("e_w"),
            ) {
                return Ok(Coordinate {
                    lat: if CompassDirection::from(n_s.as_str()) == CompassDirection::North {
                        1.
                    } else {
                        -1.
                    } * Coordinate::sexagesimal_to_decimal(
                        lat_deg.as_str().parse()?,
                        match lat_min {
                            None => None,
                            Some(lat_min) => Some(lat_min.as_str().parse()?),
                        },
                        match lat_sec {
                            None => None,
                            Some(lat_min) => Some(lat_min.as_str().parse()?),
                        },
                    ),
                    lng: if CompassDirection::from(e_w.as_str()) == CompassDirection::East {
                        1.
                    } else {
                        -1.
                    } * Coordinate::sexagesimal_to_decimal(
                        long_deg.as_str().parse()?,
                        match long_min {
                            None => None,
                            Some(long_min) => Some(long_min.as_str().parse()?),
                        },
                        match long_sec {
                            None => None,
                            Some(long_min) => Some(long_min.as_str().parse()?),
                        },
                    ),
                });
            }
        }
        Err(CoordinateError::Malformed)
    }
}

impl FromStr for Coordinate {
    type Err = CoordinateError;

    fn from_str(str_coords: &str) -> Result<Self, Self::Err> {
        let mut result = Err(CoordinateError::MissingParser);

        #[cfg(feature = "parse_str_dd")]
        {
            result = result.or_else(|_| Self::parse_str_dd(str_coords));
        }
        #[cfg(feature = "parse_str_dms")]
        {
            result = result.or_else(|_| Self::parse_str_dms(str_coords));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "parse_str_dd")]
    #[test]
    fn parse_str_dd_integer() {
        use crate::Coordinate;

        let expected = Ok(Coordinate { lat: 10., lng: 20. });
        let real = Coordinate::parse_str_dd("10,20");
        assert_eq!(expected, real);
    }
    #[cfg(feature = "parse_str_dd")]
    #[test]
    fn parse_str_dd_float() {
        use crate::Coordinate;

        let expected = Ok(Coordinate { lat: 10., lng: 20. });
        let real = Coordinate::parse_str_dd("10.0,20.0");
        assert_eq!(expected, real);
    }
    #[cfg(feature = "parse_str_dd")]
    #[test]
    fn parse_str_dd_invalid() {
        use crate::{Coordinate, CoordinateError};

        let expected = Err(CoordinateError::Malformed);
        let real = Coordinate::parse_str_dd("Asd,20.0");
        assert_eq!(expected, real);
    }
}

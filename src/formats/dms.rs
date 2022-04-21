use core::fmt;
use std::{fmt::Display, str::FromStr};

use crate::{Coordinate, CoordinateError};
use regex::Regex;

/// Compass Direction on the horizontal axis
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompassHorizontalDirection {
    /// Direction west
    West,
    /// Direction East
    East,
}

impl TryFrom<&str> for CompassHorizontalDirection {
    type Error = CoordinateError;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        match str {
            "e" | "E" => Ok(Self::East),
            "w" | "W" => Ok(Self::West),
            _ => Err(CoordinateError::InvalidValue),
        }
    }
}

impl From<CompassHorizontalDirection> for f64 {
    fn from(dir: CompassHorizontalDirection) -> f64 {
        match dir {
            CompassHorizontalDirection::East => 1.,
            CompassHorizontalDirection::West => -1.,
        }
    }
}

impl From<f64> for CompassHorizontalDirection {
    fn from(dir: f64) -> CompassHorizontalDirection {
        if dir < 0. {
            CompassHorizontalDirection::West
        } else {
            CompassHorizontalDirection::East
        }
    }
}

/// Compass Direction on the vertical axis
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompassVerticalDirection {
    /// Direction North
    North,
    /// Direction South
    South,
}

impl TryFrom<&str> for CompassVerticalDirection {
    type Error = CoordinateError;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        match str {
            "n" | "N" => Ok(Self::North),
            "s" | "S" => Ok(Self::South),
            _ => Err(CoordinateError::InvalidValue),
        }
    }
}

impl From<CompassVerticalDirection> for f64 {
    fn from(dir: CompassVerticalDirection) -> f64 {
        match dir {
            CompassVerticalDirection::North => 1.,
            CompassVerticalDirection::South => -1.,
        }
    }
}

impl From<f64> for CompassVerticalDirection {
    fn from(dir: f64) -> CompassVerticalDirection {
        if dir < 0. {
            CompassVerticalDirection::South
        } else {
            CompassVerticalDirection::North
        }
    }
}
/// A Degree, Minute, Second unit for dms coordinates
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
struct DMSUnit {
    /// Degrees of the unit
    degrees: f64,
    /// Minutes of the unit
    minutes: f64,
    /// Seconds of the unit
    seconds: f64,
}

impl From<DMSUnit> for f64 {
    fn from(dms: DMSUnit) -> f64 {
        dms.degrees + dms.minutes / 60. + dms.seconds / 60. / 60.
    }
}

impl From<f64> for DMSUnit {
    fn from(float: f64) -> Self {
        Self {
            degrees: float.abs().floor(),
            minutes: (float.abs().fract() * 60.).floor() / 60.,
            seconds: (float.abs().fract() * 60. * 60.).floor() / 60. / 60.,
        }
    }
}

/// A Coordinate in the floating point representation
/// (e.g. 12.345,6.789)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DMSCoordinate {
    /// Position on the east_west / horizontal axis
    east_west: (DMSUnit, CompassHorizontalDirection),
    /// Position on the north_south / vertical axis
    north_south: (DMSUnit, CompassVerticalDirection),
}

impl FromStr for DMSCoordinate {
    type Err = CoordinateError;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let trans_str_coords = str.to_uppercase();
        let trans_str_coords = trans_str_coords.trim();
        let long_lat_re = Regex::new("^(?P<lat_deg>\\d+(\\.\\d+)?)°((?P<lat_min>\\d+(\\.\\d+)?)')?((?P<lat_sec>\\d+(\\.\\d+)?)\"?)(?P<n_s>[NS])\\s*(?P<long_deg>\\d+(\\.\\d+)?)°((?P<long_min>\\d+(\\.\\d+)?)')?((?P<long_sec>\\d+(\\.\\d+)?)\")?(?P<e_w>[EW])$").unwrap();
        let re_captures = long_lat_re.captures(trans_str_coords);
        if let Some(captures) = re_captures {
            if let (
                Some(lat_deg),
                lat_min,
                lat_sec,
                Some(n_s),
                Some(lng_deg),
                lng_min,
                lng_sec,
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
                return Ok(DMSCoordinate {
                    north_south: (
                        DMSUnit {
                            degrees: lat_deg.as_str().parse()?,
                            minutes: lat_min
                                .map(|lat_min| lat_min.as_str().parse())
                                .unwrap_or(Ok(0.0))?,
                            seconds: lat_sec
                                .map(|lat_sec| lat_sec.as_str().parse())
                                .unwrap_or(Ok(0.0))?,
                        },
                        CompassVerticalDirection::try_from(n_s.as_str())?,
                    ),
                    east_west: (
                        DMSUnit {
                            degrees: lng_deg.as_str().parse()?,
                            minutes: lng_min
                                .map(|lng_min| lng_min.as_str().parse())
                                .unwrap_or(Ok(0.0))?,
                            seconds: lng_sec
                                .map(|lng_sec| lng_sec.as_str().parse())
                                .unwrap_or(Ok(0.0))?,
                        },
                        CompassHorizontalDirection::try_from(e_w.as_str())?,
                    ),
                });
            }
        }
        Err(CoordinateError::Malformed)
    }
}

impl Display for DMSCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lat_deg = self.north_south.0.degrees.abs().to_string() + "°";
        let lat_min = if self.north_south.0.minutes == 0. {
            "".to_string()
        } else {
            self.north_south.0.minutes.to_string() + "'"
        };
        let lat_sec = if self.north_south.0.seconds == 0. {
            "".to_string()
        } else {
            self.north_south.0.seconds.to_string() + "'"
        };
        let lng_deg = self.east_west.0.degrees.abs().to_string() + "°";
        let lng_min = if self.east_west.0.minutes == 0. {
            "".to_string()
        } else {
            self.east_west.0.minutes.to_string() + "'"
        };
        let lng_sec = if self.east_west.0.seconds == 0. {
            "".to_string()
        } else {
            self.east_west.0.seconds.to_string() + "'"
        };
        write!(
            f,
            "{}{}{}{},{}{}{}{}",
            lat_deg,
            lat_min,
            lat_sec,
            f64::from(self.north_south.1),
            lng_deg,
            lng_min,
            lng_sec,
            f64::from(self.east_west.1),
        )
    }
}

impl From<DMSCoordinate> for Coordinate {
    fn from(dd_coord: DMSCoordinate) -> Self {
        Self {
            lat: f64::from(dd_coord.north_south.0) * f64::from(dd_coord.north_south.1),
            lng: f64::from(dd_coord.east_west.0) * f64::from(dd_coord.east_west.1),
        }
    }
}

impl From<Coordinate> for DMSCoordinate {
    fn from(coord: Coordinate) -> Self {
        Self {
            north_south: (
                DMSUnit::from(coord.lat),
                CompassVerticalDirection::from(coord.lat),
            ),
            east_west: (
                DMSUnit::from(coord.lng),
                CompassHorizontalDirection::from(coord.lng),
            ),
        }
    }
}

use std::{fmt, fmt::Display, str::FromStr};

use crate::{Coordinate, CoordinateError};

/// The Geohash allows you to describe a rect on the globa.
/// It's made up by the top left and bottom right corner of the bounding rect.
/// If you want to know the center, use the center() function.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Geohash {
    /// Top left bounds of rect
    bounding_top_left: Coordinate,
    /// Top bottom right of rect
    bounding_bottom_right: Coordinate,
}

impl Geohash {
    /// Provides the center of the Geohash rect
    pub fn center(&self) -> Coordinate {
        Coordinate {
            lat: (self.bounding_top_left.lat + self.bounding_bottom_right.lat) / 2.,
            lng: (self.bounding_top_left.lng + self.bounding_bottom_right.lng) / 2.,
        }
    }
}

impl Default for Geohash {
    fn default() -> Self {
        Self {
            bounding_top_left: Coordinate {
                lat: 90.,
                lng: -180.,
            },
            bounding_bottom_right: Coordinate {
                lat: -90.,
                lng: 180.,
            },
        }
    }
}

impl From<Coordinate> for Geohash {
    fn from(coord: Coordinate) -> Self {
        Geohash {
            bounding_top_left: coord.clone(),
            bounding_bottom_right: coord,
        }
    }
}

impl From<Geohash> for Coordinate {
    fn from(hash: Geohash) -> Self {
        hash.center()
    }
}

impl FromStr for Geohash {
    type Err = CoordinateError;
    /// Parse a provided geohash
    ///
    /// ```
    /// # use recoord::formats::geohash::Geohash;
    /// # use std::str::FromStr;
    /// let hash = "ezs42";
    /// let geohash = Geohash::from_str(hash);
    /// assert!(geohash.is_ok());
    /// ```
    fn from_str(str_hash: &str) -> Result<Self, Self::Err> {
        let b32s = str_hash.chars().map(GeohashB32::try_from);
        let first_bit_lat = str_hash.chars().count() % 2;
        let first_bits_lat = [0, 1].iter().cycle().skip(first_bit_lat);

        b32s.zip(first_bits_lat)
            .try_fold(Geohash::default(), |acc, (b32, first_bit_lat)| {
                b32.map(|b32| {
                    let mut res = acc.clone();
                    for i in (0..=4).rev() {
                        let bit = (b32.0 >> i) & 0b1;
                        if (i + first_bit_lat) % 2 == 0 {
                            res.bounding_top_left.lat /= 2. * bit as f64;
                            res.bounding_bottom_right.lat /= 2. * bit as f64;
                        } else {
                            res.bounding_top_left.lng /= 2. * bit as f64;
                            res.bounding_bottom_right.lng /= 2. * bit as f64;
                        }
                    }
                    res
                })
            })
    }
}

impl Display for Geohash {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        let _center = self.center();
        let _accuracy = (
            (self.bounding_top_left.lat - self.bounding_bottom_right.lat) / 2.,
            (self.bounding_bottom_right.lng - self.bounding_top_left.lng) / 2.,
        );

        unimplemented!("Converting Geohashes to Strings is still pending - sorry")
    }
}

/// A geohash character
///
/// The geohash alphabet for mapping hash chars to values (index is value)
/// 01234 56789 bcdef ghjkm npqrs tuvwx yz
#[derive(Debug)]
struct GeohashB32(u8);

impl TryFrom<char> for GeohashB32 {
    type Error = CoordinateError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        Ok(GeohashB32(match c.to_ascii_lowercase() {
            '0'..='9' => c as u32 - '0' as u32,
            'a' => return Err(Self::Error::InvalidValue),
            'b'..='h' => c as u32 - 'b' as u32 + 10,
            'i' => return Err(Self::Error::InvalidValue),
            'j' | 'k' => c as u32 - 'j' as u32 + 17,
            'l' => return Err(Self::Error::InvalidValue),
            'm' | 'n' => c as u32 - 'm' as u32 + 19,
            'o' => return Err(Self::Error::InvalidValue),
            'p'..='z' => c as u32 - 'p' as u32 + 21,
            _ => return Err(Self::Error::InvalidValue),
        } as u8))
    }
}

impl From<GeohashB32> for char {
    fn from(ghb: GeohashB32) -> char {
        match ghb.0 {
            0..=9 => char::from_digit(ghb.0 as u32, 10).unwrap(),
            10..=17 => (b'b' + ghb.0 - 10) as char,
            18..=19 => (b'j' + ghb.0 - 18) as char,
            20..=21 => (b'm' + ghb.0 - 20) as char,
            22..=32 => (b'p' + ghb.0 - 22) as char,
            _ => unreachable!(),
        }
    }
}

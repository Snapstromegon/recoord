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

    /// Height of the bounding rect
    pub fn height(&self) -> f64 {
        self.bounding_bottom_right.lat - self.bounding_top_left.lat
    }

    /// Width of the bounding rect
    pub fn width(&self) -> f64 {
        self.bounding_bottom_right.lng - self.bounding_top_left.lng
    }

    fn min_chars_for_precision(bits: usize) -> usize {
        /*
            This is correct, since each char represents 5 bits.
            Intuitively you would use n/5 for n bits, but this creatres an off by one error,
            which is resolved by the -1 and +1.

                        +-----------------------------+
            Bits        | 1| 2| 3| 4| 5| 6| 7| 8| 9|10|
                        +-----------------------------+
            Expected    | 1| 1| 1| 1| 1| 2| 2| 2| 2| 2|
                        +-----------------------------+
            bits/5      | 0| 0| 0| 0| 1| 1| 1| 1| 1| 1|
            bits/5+1    | 1| 1| 1| 1| 2| 2| 2| 2| 2| 3|
            (bits-1)/5+1| 1| 1| 1| 1| 1| 2| 2| 2| 2| 2|
                        +-----------------------------+
        */
        (bits - 1) / 5 + 1
    }

    /// Create a hash string to a specific bits precision
    pub fn hash_with_precision(&self, bits: usize) -> String {
        self.hash_with_max_length(Geohash::min_chars_for_precision(bits))
    }

    fn _hash_with_bits(&self, lat_bits: usize, lng_bits: usize) -> Result<String, CoordinateError> {
        let total_bits = lat_bits + lng_bits;
        let char_count = total_bits / 5;
        let is_first_bit_lat = char_count % 2 == 0;
        let bits_fit_in_char = total_bits % 5 == 0;
        let bits_correct_ratio = if total_bits % 10 == 0 {
            lat_bits == lng_bits
        } else {
            lat_bits == lng_bits + 1
        };
        
        if bits_fit_in_char && bits_correct_ratio {
            let lat =
                (self.center().lat * (2u64.pow(lat_bits as u32) as f64) / 180.).round() as usize;
            let lng =
                (self.center().lng * (2u64.pow(lng_bits as u32) as f64) / 360.).round() as usize;

            let lat_bits = (0..lat_bits)
                .rev()
                .map(|i| (lat >> i) & 0b1)
                .collect::<Vec<usize>>()
                .into_iter();
            let lng_bits = (0..lng_bits)
                .rev()
                .map(|i| (lng >> i) & 0b1)
                .collect::<Vec<usize>>()
                .into_iter();

            // let is_lat = [is_first_bit_lat, !is_first_bit_lat];
            // let is_lat = is_lat.into_iter().cycle();

            let (odd_bits, even_bits) = if is_first_bit_lat {
                (lat_bits, lng_bits)
            } else {
                (lng_bits, lat_bits)
            };

            let all_bits: Vec<usize> = odd_bits
                .zip(even_bits)
                .map(|(a, b)| [a, b])
                .flatten()
                // .zip(is_lat)
                .collect();

            let mut res = "".to_string();

            for chunk in all_bits.chunks(5) {
                let mut byte = 0;
                for (i, value) in chunk.into_iter().enumerate() {
                    byte |= value << (4 - i);
                }
                res.push(char::try_from(GeohashB32(byte as u8))?);
            }

            Ok(res)
        } else {
            Err(CoordinateError::Malformed)
        }
    }

    /// Create a hash with a specified number of characters
    pub fn hash_with_max_length(&self, _length: usize) -> String {
        // let width_divisions = 360. / self.width();
        // let height_divisions = 180. / self.height();

        // let width_bits = width_divisions.log2();
        // let height_bits = height_divisions.log2();

        // let center = self.center();

        // let height_index = center.lat + 90. / height_divisions;
        // let width_index = center.lng + 180. / width_divisions;

        unimplemented!()
    }

    /// Create the smallest hash, that includes top_left and bottom_right
    pub fn get_inner_hash(&self) -> String {
        unimplemented!()
    }

    /// Create the largest hash, that does nto includes top_left and bottom_right
    pub fn get_outer_hash(&self) -> String {
        unimplemented!()
    }

    /// Create the hash that has the biggest match with the described area
    pub fn get_closest_hash(&self) -> String {
        unimplemented!()
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
        // Open Questions:
        //  - Is this possible in a fairly (really) performant way (should be, I think)?
        //  - Which precision should be used here?
        //      - We could make it max precision (which might be actually fastest)
        //      - Make best effort to shortening (what about not correctly aligned hashes) -> inner or outer bounds?
        //  - Should there be an extra function to stringify a specific precision and inner/outer/most matching bounds?
        //
        // Personally I think it should return the shortest outer bound hash.
        // This tends to return short hashes and guarantees to include the whole described region.
        // Only downside is, that you loose precision

        let _center = self.center();
        let _accuracy = (
            (self.bounding_top_left.lat - self.bounding_bottom_right.lat) / 2.,
            (self.bounding_bottom_right.lng - self.bounding_top_left.lng) / 2.,
        );

        todo!()
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

// impl From<GeohashB32> for char {
//     fn from(ghb: GeohashB32) -> char {
//         match ghb.0 {
//             0..=9 => char::from_digit(ghb.0 as u32, 10).unwrap(),
//             10..=17 => (b'b' + ghb.0 - 10) as char,
//             18..=19 => (b'j' + ghb.0 - 18) as char,
//             20..=21 => (b'm' + ghb.0 - 20) as char,
//             22..=32 => (b'p' + ghb.0 - 22) as char,
//             _ => unreachable!(),
//         }
//     }
// }

impl TryFrom<GeohashB32> for char {
    type Error = CoordinateError;
    fn try_from(ghb: GeohashB32) -> Result<char, Self::Error> {
        Ok(match ghb.0 {
            0..=9 => char::from_digit(ghb.0 as u32, 10).unwrap(),
            10..=17 => (b'b' + ghb.0 - 10) as char,
            18..=19 => (b'j' + ghb.0 - 18) as char,
            20..=21 => (b'm' + ghb.0 - 20) as char,
            22..=32 => (b'p' + ghb.0 - 22) as char,
            _ => return Err(Self::Error::InvalidValue),
        })
    }
}

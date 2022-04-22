use std::{fmt, fmt::Display, str::FromStr};

use crate::{Coordinate, CoordinateError};

/// The Geohash allows you to describe a rect on the globa.
/// It's made up by the top left and bottom right corner of the bounding rect.
/// If you want to know the center, use the center() function.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
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

    /// Encode a hash with a given precision in bits
    ///
    /// ```
    /// # use recoord::formats::geohash::*;
    /// let input = "1";
    /// let h: Result<Geohash, _> = input.parse();
    /// assert!(h.is_ok());
    /// let h= h.unwrap();
    /// println!("{h:?}");
    /// let c = h.center();
    /// println!("{c}");
    /// let s = h.hash_with_max_length(input.chars().count());
    /// assert_eq!(s, input);
    /// ```
    pub fn hash_with_precision(&self, total_bits: usize) -> Result<String, CoordinateError> {
        let lat_bits = total_bits / 2;
        let lng_bits = total_bits / 2 + total_bits % 2;
        let bits_fit_in_char = total_bits % 5 == 0;
        let bits_correct_ratio = if total_bits % 10 == 0 {
            lat_bits == lng_bits
        } else {
            lat_bits == lng_bits - 1
        };

        if bits_fit_in_char && bits_correct_ratio {
            let cells_n_lat = 1usize << lat_bits;
            let cells_n_lng = 1usize << lng_bits;

            let lat = (90. + self.center().lat) / 180. * cells_n_lat as f64;
            let lng = (180. + self.center().lng) / 360. * cells_n_lng as f64;

            let lat = lat.floor() as usize;
            let lng = lng.floor() as usize;

            let lat_bits = (0..lat_bits).rev().map(|i| Some((lat >> i) & 0b1));
            let lng_bits = (0..lng_bits).rev().map(|i| Some((lng >> i) & 0b1));

            let all_bits: Vec<usize> = lng_bits
                .zip(lat_bits.chain(std::iter::repeat(None)))
                .flat_map(|(a, b)| [a, b])
                .map_while(|item| item)
                .collect();

            let mut res = "".to_string();

            for chunk in all_bits.chunks(5) {
                let mut byte = 0;
                for (i, value) in chunk.iter().enumerate() {
                    byte |= value << (4 - i);
                }
                // We know at this point, that byte only container 5 bits and therefore is safe to unwrap
                res.push(char::try_from(GeohashB32(byte as u8)).unwrap());
            }

            Ok(res)
        } else {
            Err(CoordinateError::Malformed)
        }
    }

    /// Create a hash with a specified number of characters
    pub fn hash_with_max_length(&self, length: usize) -> String {
        // The unwrap is safe, since we guarantee, that the length is a multiple of 5
        self.hash_with_precision(length * 5).unwrap()
    }

    /// Create the smallest hash, that includes top_left and bottom_right
    pub fn get_inner_hash(&self) -> String {
        let lat_bits = (360. / self.height()).ceil() as usize
            - if self.crosses_vertical_chunks() { 1 } else { 0 };
        let lng_bits = (360. / self.width()).ceil() as usize
            - if self.crosses_horizontal_chunks() {
                1
            } else {
                0
            };

        let min_bits = lat_bits.max(lng_bits);
        let needed_bits = (((min_bits - 1) / 5) + 1) * 5;
        // The line above guarantees that needed_bits is a multiple of 5
        self.hash_with_precision(needed_bits).unwrap()
    }

    /// Create the largest hash, that does nto includes top_left and bottom_right
    pub fn get_outer_hash(&self) -> String {
        let lat_bits = (360. / self.height()).floor() as usize
            - if self.crosses_vertical_chunks() { 1 } else { 0 };
        let lng_bits = (360. / self.width()).floor() as usize
            - if self.crosses_horizontal_chunks() {
                1
            } else {
                0
            };

        let max_bits = lat_bits.min(lng_bits);
        let needed_bits = (((max_bits + 1) / 5) - 1) * 5;
        // The line above guarantees that needed_bits is a multiple of 5
        self.hash_with_precision(needed_bits).unwrap()
    }

    // /// Create the hash that has the biggest match with the described area
    // pub fn get_closest_hash(&self) -> Result<String, CoordinateError> {
    //     unimplemented!()
    // }

    fn crosses_horizontal_chunks(&self) -> bool {
        let left_cell = (self.bounding_top_left.lng / self.width()).floor() as usize;
        let right_cell = (self.bounding_bottom_right.lng / self.width()).floor() as usize;
        left_cell == right_cell
    }

    fn crosses_vertical_chunks(&self) -> bool {
        let top_cell = (self.bounding_top_left.lat / self.height()).floor() as usize;
        let bottom_cell = (self.bounding_bottom_right.lat / self.height()).floor() as usize;
        top_cell == bottom_cell
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
        let first_bits_lat = [1, 0].iter().cycle();

        b32s.zip(first_bits_lat)
            .try_fold(Geohash::default(), |acc, (b32, first_bit_lat)| {
                b32.map(|b32| {
                    let mut res = acc.clone();
                    for i in (0..=4).rev() {
                        let bit = (b32.0 >> i) & 0b1;
                        if (i + first_bit_lat) % 2 == 0 {
                            let mid_lat =
                                (res.bounding_top_left.lat + res.bounding_bottom_right.lat) / 2.;
                            if bit == 0 {
                                res.bounding_top_left.lat = mid_lat;
                            } else {
                                res.bounding_bottom_right.lat = mid_lat;
                            }
                        } else {
                            let mid_lng =
                                (res.bounding_top_left.lng + res.bounding_bottom_right.lng) / 2.;
                            if bit == 0 {
                                res.bounding_bottom_right.lng = mid_lng;
                            } else {
                                res.bounding_top_left.lng = mid_lng;
                            }
                        }
                    }
                    res
                })
            })
    }
}

impl Display for Geohash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

        write!(f, "{}", self.get_outer_hash())
    }
}

/// A geohash character
///
/// The geohash alphabet for mapping hash chars to values (index is value)
/// 01234 56789 bcdef ghjkm npqrs tuvwx yz
#[derive(Debug, PartialEq, Eq)]
struct GeohashB32(u8);

impl TryFrom<char> for GeohashB32 {
    type Error = CoordinateError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        let c = c.to_ascii_lowercase() as u8;
        Ok(GeohashB32(match c {
            b'0'..=b'9' => c - b'0',
            b'a' => return Err(Self::Error::InvalidValue),
            b'b'..=b'h' => c - b'b' + 10,
            b'i' => return Err(Self::Error::InvalidValue),
            b'j' | b'k' => c - b'j' + 17,
            b'l' => return Err(Self::Error::InvalidValue),
            b'm' | b'n' => c - b'm' + 19,
            b'o' => return Err(Self::Error::InvalidValue),
            b'p'..=b'z' => c - b'p' + 21,
            _ => return Err(Self::Error::InvalidValue),
        }))
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
            10..=16 => (b'b' + ghb.0 - 10) as char,
            17..=18 => (b'j' + ghb.0 - 17) as char,
            19..=20 => (b'm' + ghb.0 - 19) as char,
            21..=32 => (b'p' + ghb.0 - 21) as char,
            _ => return Err(Self::Error::InvalidValue),
        })
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    const ALPHABET: &str = "0123456789bcdefghjkmnpqrstuvwxyz";

    #[test]
    fn test_char_to_geohashb32() {
        for (i, expected) in ALPHABET.chars().enumerate() {
            assert_eq!(GeohashB32::try_from(expected).unwrap(), GeohashB32(i as u8));
        }
    }

    #[test]
    fn test_char_to_geohashb32_errors() {
        assert!(GeohashB32::try_from('a').is_err());
        assert!(GeohashB32::try_from('ö').is_err());
        assert!(GeohashB32::try_from('💥').is_err());
    }

    #[test]
    fn test_geohashb32_to_char() {
        for (i, expected) in ALPHABET.chars().enumerate() {
            assert_eq!(char::try_from(GeohashB32(i as u8)).unwrap(), expected);
        }
    }

    #[test]
    fn test_geohash_decode_encode() {
        for expected in ALPHABET.chars() {
            let geohash = Geohash::from_str(&expected.to_string());
            assert!(geohash.is_ok());
            let geohash = geohash.unwrap();
            let result = geohash.hash_with_max_length(1);
            assert_eq!(result, expected.to_string());
        }
    }

    #[test]
    fn test_geohash_decode_encode_stresstest() {
        let hashes = build_test_hash_with_length(2, None);
        println!("hi {}", hashes.len());
        for hash in hashes {
            println!("Testing hash {hash}");
            // std::io::stdout().flush().unwrap();
            let geohash = Geohash::from_str(&hash);
            assert!(geohash.is_ok());
            let geohash = geohash.unwrap();
            let result = geohash.hash_with_max_length(hash.chars().count());
            assert_eq!(result, hash); 
        }
    }

    fn build_test_hash_with_length(length: usize, per_depth: Option<usize>) -> Vec<String> {
        match length {
            0 => vec![],
            1 => ALPHABET.chars().map(|c| c.to_string()).collect(),
            _ => {
                let sub_hashes = build_test_hash_with_length(length - 1, per_depth);
                let mut res = vec![];
                for possible in ALPHABET.chars().take(per_depth.unwrap_or(32)) {
                    res.push(possible.to_string());
                    for sub_hash in sub_hashes.iter() {
                        let mut res_str = possible.to_string();
                        res_str.push_str(&sub_hash);
                        res.push(res_str);
                    }
                }
                res
            }
        }
    }
}

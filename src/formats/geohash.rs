use crate::{Coordinate, CoordinateError};


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
            10..=17 => ('b' as u8 + ghb.0 - 10) as char,
            18..=19 => ('j' as u8 + ghb.0 - 18) as char,
            20..=21 => ('m' as u8 + ghb.0 - 20) as char,
            22..=32 => ('p' as u8 + ghb.0 - 22) as char,
            _ => unreachable!()
        }
    }
}

impl GeohashB32 {
    /// Get the values of the even bits in the char
    pub fn get_even_bits_value(&self) -> Vec<bool> {
        let mut result = vec![];
        for i in (0..2).rev() {
            let bit_of_interest = 0b1 << (2 * i + 1);
            result.push((self.0 & bit_of_interest) >> (i + 1) != 0);
        }
        result
    }

    /// Get the values of the odd bits in the char
    pub fn get_odd_bits_value(&self) -> Vec<bool> {
        let mut result = vec![];
        for i in (0..3).rev() {
            let bit_of_interest = 0b1 << (2 * i);
            result.push((self.0 & bit_of_interest) >> (i) != 0);
        }
        result
    }

    /// Get the latitude and longitude bits of a char
    pub fn get_lat_lng_bit_values(&self, first_bit_lat: bool) -> (Vec<bool>, Vec<bool>) {
        if first_bit_lat {
            (self.get_odd_bits_value(), self.get_even_bits_value())
        } else {
            (self.get_even_bits_value(), self.get_odd_bits_value())
        }
    }
}

// impl TryFrom<char> for GeohashB32 {
//     type Error = CoordinateError;
//     fn try_from(c: char) -> Result<Self, Self::Error> {
//         GEOHASH_BASE32_ALPHABET
//             .chars()
//             .position(|char| char == c)
//             .map_or(Err(Self::Error::Malformed), |pos| Ok(Self(pos as u8)))
//     }
// }

/// Parse a provided geohash
///
/// ```
/// # use recoord::geohash::parse_geohash;
/// let hash = "ezs42";
/// let geohash = parse_geohash(hash);
/// assert!(geohash.is_ok());
/// ```
pub fn parse_geohash(hash: &str) -> Result<Coordinate, CoordinateError> {
    let lowercased = hash.to_lowercase();
    let first_bit_lat = lowercased.chars().count() % 2;
    let first_bits_lat = [true, false].iter().cycle().skip(first_bit_lat);

    let values = lowercased
        .chars()
        .map(GeohashB32::try_from)
        .zip(first_bits_lat)
        .map(|(geohash, first_bit_lat)| geohash.map(|gh| gh.get_lat_lng_bit_values(*first_bit_lat)))
        .fold(Ok((vec![], vec![])), |acc, curr| match (acc, curr) {
            (Err(e), _) | (_, Err(e)) => Err(e),
            (Ok((acc_lat, acc_lng)), Ok((curr_lat, curr_lng))) => {
                Ok(([acc_lat, curr_lat].concat(), [acc_lng, curr_lng].concat()))
            }
        })?;

    Coordinate::try_from((
        bool_vec_to_scaled_number(&values.0, -90.0, 90.0),
        bool_vec_to_scaled_number(&values.1, -180.0, 180.0),
    ))
}

/// Map a boolean vector to a scale between min and max
fn bool_vec_to_scaled_number(bools: &[bool], min: f64, max: f64) -> f64 {
    let range_len = max - min;
    let mut result = 0.0;
    for (i, bit) in bools.iter().enumerate() {
        let current_value = range_len / 2u64.pow(i as u32 + 1) as f64;
        result += current_value * if *bit { 1.0 } else { 0.0 };
    }
    result + min
}

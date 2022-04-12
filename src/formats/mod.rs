/// Double floats format (12.345,67.890)
#[cfg(feature = "format_dd")]
pub mod dd;
/// Degree, Minutes, Seconds format (12°34'56"N 9°12'23"E)
#[cfg(feature = "format_dms")]
pub mod dms;
/// Geohash format (ezs42)
#[cfg(feature = "format_geohash")]
pub mod geohash;

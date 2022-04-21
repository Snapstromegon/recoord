use crate::{Coordinate, CoordinateError};
use serde::Deserialize;

/// Location of Open Street Maps
#[derive(Deserialize)]
struct OSMLocation {
    /// Latitude
    lat: String,
    /// Longitude
    lon: String,
}

/// Resolve a location synchronously name to a coordinate
pub fn resolve_sync(location: &str) -> Result<Coordinate, CoordinateError> {
    let locations = reqwest::blocking::Client::new()
        .get("https://nominatim.openstreetmap.org/search")
        .header(reqwest::header::USER_AGENT, "tanker_price")
        .query(&[("format", "json"), ("q", location)])
        .send()?
        .json::<Vec<OSMLocation>>()?;
    if let Some(location) = locations.get(0) {
        Ok(Coordinate {
            lng: location.lon.parse()?,
            lat: location.lat.parse()?,
        })
    } else {
        Err(CoordinateError::Unresolveable)
    }
}

/// Resolve a location name to a coordinate
pub async fn resolve(location: &str) -> Result<Coordinate, CoordinateError> {
    let locations = reqwest::Client::new()
        .get("https://nominatim.openstreetmap.org/search")
        .header(reqwest::header::USER_AGENT, "tanker_price")
        .query(&[("format", "json"), ("q", location)])
        .send()
        .await?
        .json::<Vec<OSMLocation>>()
        .await?;
    if let Some(location) = locations.get(0) {
        Ok(Coordinate {
            lng: location.lon.parse()?,
            lat: location.lat.parse()?,
        })
    } else {
        Err(CoordinateError::Unresolveable)
    }
}

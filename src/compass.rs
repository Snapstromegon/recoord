/// Possible compass directions
#[derive(PartialEq, Eq, Debug)]
pub enum CompassDirection {
    /// North
    North,
    /// East
    East,
    /// South
    South,
    /// West
    West,
}

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

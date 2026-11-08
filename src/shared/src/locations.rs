use strum_macros::Display;

/// Cities/towns in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Location {
    Brandenburg,
    EchoCreek,
    Ekron,
    FallasLake,
    Irvington,
    MarchRidge,
    Muldraugh,
    Riverside,
    Rosewood,
    ValleyStation,
    WestPoint,
}

impl Location {
    /// Matches the enum variant to the name in "media/maps" folder
    pub fn from_folder_name(folder_name: &str) -> Option<Self> {
        match folder_name {
            "Brandenburg, KY" => Some(Location::Brandenburg),
            "Echo Creek, KY" => Some(Location::EchoCreek),
            "Ekron, KY" => Some(Location::Ekron),
            "Fallas Lake, KY" => Some(Location::FallasLake),
            "Irvington, KY" => Some(Location::Irvington),
            "March Ridge, KY" => Some(Location::MarchRidge),
            "Muldraugh, KY" => Some(Location::Muldraugh),
            "Riverside, KY" => Some(Location::Riverside),
            "Rosewood, KY" => Some(Location::Rosewood),
            "Valley Station, KY" => Some(Location::ValleyStation),
            "West Point, KY" => Some(Location::WestPoint),
            _ => None,
        }
    }
}
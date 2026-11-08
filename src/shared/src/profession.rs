use strum_macros::Display;

/// A character's profession
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Profession {
    Unemployed,
    Blacksmith,
    BurgerFlipper,
    Burglar,
    Carpenter,
    Chef,
    ConstructionWorker,
    DIYExpert,
    Doctor,
    Electrician,
    Engineer,
    Farmer,
    Firefighter,
    FishingGuide,
    FitnessInstructor,
    Lumberjack,
    Mechanic,
    Nurse,
    ParkRanger,
    PoliceOfficer,
    Rancher,
    SecurityGuard,
    Tailor,
    Veteran,
    Welder,
}

impl Profession {
    pub fn from_id(profession_str: &str) -> Option<Self> {
        match profession_str {
            "burgerflipper" => Some(Profession::BurgerFlipper),
            "burglar" => Some(Profession::Burglar),
            "carpenter" => Some(Profession::Carpenter),
            "chef" => Some(Profession::Chef),
            "constructionworker" => Some(Profession::ConstructionWorker),
            "doctor" => Some(Profession::Doctor),
            "electrician" => Some(Profession::Electrician),
            "engineer" => Some(Profession::Engineer),
            "farmer" => Some(Profession::Farmer),
            "fireofficer" => Some(Profession::Firefighter),
            "fisherman" => Some(Profession::FishingGuide),
            "fitnessInstructor" => Some(Profession::FitnessInstructor),
            "lumberjack" => Some(Profession::Lumberjack),
            "mechanics" => Some(Profession::Mechanic),
            "metalworker" => Some(Profession::Welder),
            "nurse" => Some(Profession::Nurse),
            "parkranger" => Some(Profession::ParkRanger),
            "policeofficer" => Some(Profession::PoliceOfficer),
            "rancher" => Some(Profession::Rancher),
            "repairman" => Some(Profession::DIYExpert),
            "securityguard" => Some(Profession::SecurityGuard),
            "smither" => Some(Profession::Blacksmith),
            "tailor" => Some(Profession::Tailor),
            "veteran" => Some(Profession::Veteran),
            "unemployed" => Some(Profession::Unemployed),
            "veteran" => Some(Profession::Veteran),
            _ => None,
        }
    }
}
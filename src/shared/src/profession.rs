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
            _ => None,
        }
    }

    pub fn to_id(&self) -> &'static str {
        match self {
            Profession::Unemployed => "unemployed",
            Profession::Blacksmith => "smither",
            Profession::BurgerFlipper => "burgerflipper",
            Profession::Burglar => "burglar",
            Profession::Carpenter => "carpenter",
            Profession::Chef => "chef",
            Profession::ConstructionWorker => "constructionworker",
            Profession::DIYExpert => "repairman",
            Profession::Doctor => "doctor",
            Profession::Electrician => "electrician",
            Profession::Engineer => "engineer",
            Profession::Farmer => "farmer",
            Profession::Firefighter => "fireofficer",
            Profession::FishingGuide => "fisherman",
            Profession::FitnessInstructor => "fitnessInstructor",
            Profession::Lumberjack => "lumberjack",
            Profession::Mechanic => "mechanics",
            Profession::Nurse => "nurse",
            Profession::ParkRanger => "parkranger",
            Profession::PoliceOfficer => "policeofficer",
            Profession::Rancher => "rancher",
            Profession::SecurityGuard => "securityguard",
            Profession::Tailor => "tailor",
            Profession::Veteran => "veteran",
            Profession::Welder => "metalworker",
        }
    }
}

//! World types shared across modules

use serde::{Serialize, Deserialize};
use crate::math::{BlockPos, ChunkPos};
use crate::block::BlockId;

/// World height limits
pub const WORLD_HEIGHT: i32 = 256;
pub const SEA_LEVEL: i32 = 64;

/// Biome types (including dimension-specific biomes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    // Overworld biomes
    Plains,
    Forest,
    Desert,
    Mountains,
    Taiga,
    SnowyTundra,
    Jungle,
    Swamp,
    Beach,
    Ocean,
    DeepOcean,
    River,
    Savanna,
    Badlands,
    // Moon biomes
    LunarPlains,
    LunarCrater,
    LunarHighland,
    // Mars biomes
    MartianPlains,
    MartianCanyon,
    Volcanic,
    // Venus biomes
    VenusianLowlands,
    VenusianHighlands,
    SulphurSea,
    // Other celestial biomes
    MercurianPlain,
    JovianStorm,
    SaturnRing,
    SaturnCloud,
    NeptunianCore,
    PlutonianIcePlain,
    Asteroid,
    Void,
    CrystalForest,
    EmberPlains,
    FrostWastes,
    Mushroom,
}

impl Biome {
    pub fn all() -> [Biome; 35] {
        [
            // Overworld
            Biome::Plains,
            Biome::Forest,
            Biome::Desert,
            Biome::Mountains,
            Biome::Taiga,
            Biome::SnowyTundra,
            Biome::Jungle,
            Biome::Swamp,
            Biome::Beach,
            Biome::Ocean,
            Biome::DeepOcean,
            Biome::River,
            Biome::Savanna,
            Biome::Badlands,
            // Moon
            Biome::LunarPlains,
            Biome::LunarCrater,
            Biome::LunarHighland,
            // Mars
            Biome::MartianPlains,
            Biome::MartianCanyon,
            Biome::Volcanic,
            // Venus
            Biome::VenusianLowlands,
            Biome::VenusianHighlands,
            Biome::SulphurSea,
            // Others
            Biome::MercurianPlain,
            Biome::JovianStorm,
            Biome::SaturnRing,
            Biome::SaturnCloud,
            Biome::NeptunianCore,
            Biome::PlutonianIcePlain,
            Biome::Asteroid,
            Biome::Void,
            Biome::CrystalForest,
            Biome::EmberPlains,
            Biome::FrostWastes,
            Biome::Mushroom,
        ]
    }

    pub fn is_celestial(&self) -> bool {
        matches!(self,
            Biome::LunarPlains | Biome::LunarCrater | Biome::LunarHighland |
            Biome::MartianPlains | Biome::MartianCanyon | Biome::Volcanic |
            Biome::VenusianLowlands | Biome::VenusianHighlands | Biome::SulphurSea |
            Biome::MercurianPlain | Biome::JovianStorm | Biome::SaturnRing |
            Biome::SaturnCloud | Biome::NeptunianCore | Biome::PlutonianIcePlain |
            Biome::Asteroid | Biome::Void | Biome::CrystalForest |
            Biome::EmberPlains | Biome::FrostWastes
        )
    }

    pub fn temperature(&self) -> f32 {
        match *self {
            Biome::Desert => 2.0,
            Biome::Savanna => 1.5,
            Biome::Plains => 1.0,
            Biome::Forest => 0.8,
            Biome::Swamp => 0.7,
            Biome::Beach => 0.6,
            Biome::River => 0.5,
            Biome::Jungle => 1.2,
            Biome::Mountains => 0.3,
            Biome::Taiga => -0.2,
            Biome::SnowyTundra => -0.5,
            Biome::Ocean => 0.3,
            Biome::DeepOcean => 0.2,
            Biome::Badlands => 1.8,
            // Celestial bodies - extreme temperatures
            Biome::LunarPlains | Biome::LunarCrater | Biome::LunarHighland => -0.8,
            Biome::MartianPlains | Biome::MartianCanyon => -0.3,
            Biome::Volcanic | Biome::SulphurSea => 2.5,
            Biome::VenusianLowlands | Biome::VenusianHighlands => 1.8,
            Biome::MercurianPlain => 2.0,
            Biome::JovianStorm => 0.5,
            Biome::SaturnRing | Biome::SaturnCloud => -0.4,
            Biome::NeptunianCore => 0.2,
            Biome::PlutonianIcePlain => -1.0,
            Biome::Asteroid => -0.9,
            Biome::Void => 0.0,
            Biome::CrystalForest => 0.6,
            Biome::EmberPlains => 1.5,
            Biome::FrostWastes => -1.2,
            Biome::Mushroom => 0.6,
        }
    }

    pub fn humidity(&self) -> f32 {
        match *self {
            Biome::Swamp => 1.0,
            Biome::Jungle => 0.9,
            Biome::Ocean => 0.8,
            Biome::DeepOcean => 0.9,
            Biome::River => 0.6,
            Biome::Beach => 0.4,
            Biome::Forest => 0.5,
            Biome::Taiga => 0.4,
            Biome::Plains => 0.3,
            Biome::Savanna => 0.2,
            Biome::Desert => 0.0,
            Biome::Mountains => 0.3,
            Biome::SnowyTundra => 0.3,
            Biome::Badlands => 0.1,
            // Celestial bodies - typically very dry
            Biome::LunarPlains | Biome::LunarCrater | Biome::LunarHighland => 0.0,
            Biome::MartianPlains | Biome::MartianCanyon => 0.0,
            Biome::Volcanic | Biome::SulphurSea => 0.0,
            Biome::VenusianLowlands | Biome::VenusianHighlands => 0.1,
            Biome::MercurianPlain => 0.0,
            Biome::JovianStorm => 0.3,
            Biome::SaturnRing | Biome::SaturnCloud => 0.1,
            Biome::NeptunianCore => 0.2,
            Biome::PlutonianIcePlain => 0.0,
            Biome::Asteroid => 0.0,
            Biome::Void => 0.0,
            Biome::CrystalForest => 0.2,
            Biome::EmberPlains => 0.0,
            Biome::FrostWastes => 0.1,
            Biome::Mushroom => 0.7,
        }
    }
}

/// World seed for procedural generation
#[derive(Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(12345)
    }
}

/// World identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldId(pub String);

impl Default for WorldId {
    fn default() -> Self {
        Self("default".to_string())
    }
}

/// Dimension identifier for inter-dimensional travel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimensionId(pub u16);

impl DimensionId {
    pub const OVERWORLD: DimensionId = DimensionId(0);
    pub const MOON: DimensionId = DimensionId(1);
    pub const MARS: DimensionId = DimensionId(2);
    pub const VENUS: DimensionId = DimensionId(3);
    pub const MERCURY: DimensionId = DimensionId(4);
    pub const JUPITER: DimensionId = DimensionId(5);
    pub const SATURN: DimensionId = DimensionId(6);
    pub const NEPTUNE: DimensionId = DimensionId(7);
    pub const PLUTO: DimensionId = DimensionId(8);
    pub const ASTEROID_BELT: DimensionId = DimensionId(9);
    pub const THE_VOID: DimensionId = DimensionId(10);
    pub const CRYSTAL_REALM: DimensionId = DimensionId(11);
    pub const EMBER_DIMENSION: DimensionId = DimensionId(12);
    pub const FROST_REALM: DimensionId = DimensionId(13);

    pub fn name(&self) -> &'static str {
        match *self {
            DimensionId::OVERWORLD => "Overworld",
            DimensionId::MOON => "The Moon",
            DimensionId::MARS => "Mars",
            DimensionId::VENUS => "Venus",
            DimensionId::MERCURY => "Mercury",
            DimensionId::JUPITER => "Jupiter",
            DimensionId::SATURN => "Saturn",
            DimensionId::NEPTUNE => "Neptune",
            DimensionId::PLUTO => "Pluto",
            DimensionId::ASTEROID_BELT => "Asteroid Belt",
            DimensionId::THE_VOID => "The Void",
            DimensionId::CRYSTAL_REALM => "Crystal Realm",
            DimensionId::EMBER_DIMENSION => "Ember Realm",
            DimensionId::FROST_REALM => "Frost Realm",
            _ => "Unknown",
        }
    }

    pub fn description(&self) -> &'static str {
        match *self {
            DimensionId::OVERWORLD => "The familiar world you call home",
            DimensionId::MOON => "A barren satellite with low gravity and craters",
            DimensionId::MARS => "The red planet with giant volcanoes and dust storms",
            DimensionId::VENUS => "A scorching world with acidic clouds",
            DimensionId::MERCURY => "Closest to the sun with extreme temperatures",
            DimensionId::JUPITER => "A massive gas giant with extreme storms",
            DimensionId::SATURN => "Famous for its beautiful ring system",
            DimensionId::NEPTUNE => "An ice giant with supersonic winds",
            DimensionId::PLUTO => "A cold, distant world with nitrogen ice plains",
            DimensionId::ASTEROID_BELT => "Floating among ancient space rocks",
            DimensionId::THE_VOID => "An empty dimension between dimensions",
            DimensionId::CRYSTAL_REALM => "A mystical dimension of giant crystals",
            DimensionId::EMBER_DIMENSION => "A dimension of fire and lava",
            DimensionId::FROST_REALM => "An endless winter dimension",
            _ => "",
        }
    }

    pub fn required_crystal_item_id(&self) -> Option<u16> {
        match *self {
            DimensionId::MOON => Some(220),
            DimensionId::MARS => Some(221),
            DimensionId::VENUS => Some(222),
            DimensionId::MERCURY => Some(223),
            DimensionId::JUPITER => Some(224),
            DimensionId::SATURN => Some(225),
            DimensionId::NEPTUNE => Some(226),
            DimensionId::PLUTO => Some(227),
            DimensionId::ASTEROID_BELT => Some(228),
            DimensionId::THE_VOID => Some(229),
            DimensionId::CRYSTAL_REALM => Some(230),
            DimensionId::EMBER_DIMENSION => Some(231),
            DimensionId::FROST_REALM => Some(232),
            _ => None,
        }
    }

    pub fn gravity(&self) -> f32 {
        match *self {
            DimensionId::OVERWORLD => 1.0,
            DimensionId::MOON => 0.16,
            DimensionId::MARS => 0.38,
            DimensionId::VENUS => 0.9,
            DimensionId::MERCURY => 0.38,
            DimensionId::JUPITER => 2.5,
            DimensionId::SATURN => 1.1,
            DimensionId::NEPTUNE => 1.2,
            DimensionId::PLUTO => 0.06,
            DimensionId::ASTEROID_BELT => 0.02,
            DimensionId::THE_VOID => 0.0,
            DimensionId::CRYSTAL_REALM => 0.8,
            DimensionId::EMBER_DIMENSION => 1.0,
            DimensionId::FROST_REALM => 1.0,
            _ => 1.0,
        }
    }

    pub fn sky_color(&self) -> [f32; 4] {
        match *self {
            DimensionId::OVERWORLD => [0.5, 0.7, 1.0, 1.0],
            DimensionId::MOON => [0.05, 0.05, 0.1, 1.0],
            DimensionId::MARS => [0.9, 0.5, 0.3, 1.0],
            DimensionId::VENUS => [0.8, 0.6, 0.2, 1.0],
            DimensionId::MERCURY => [0.1, 0.1, 0.15, 1.0],
            DimensionId::JUPITER => [0.7, 0.6, 0.5, 1.0],
            DimensionId::SATURN => [0.75, 0.7, 0.6, 1.0],
            DimensionId::NEPTUNE => [0.2, 0.3, 0.6, 1.0],
            DimensionId::PLUTO => [0.1, 0.1, 0.2, 1.0],
            DimensionId::ASTEROID_BELT => [0.0, 0.0, 0.0, 1.0],
            DimensionId::THE_VOID => [0.0, 0.0, 0.0, 1.0],
            DimensionId::CRYSTAL_REALM => [0.3, 0.1, 0.5, 1.0],
            DimensionId::EMBER_DIMENSION => [0.4, 0.1, 0.0, 1.0],
            DimensionId::FROST_REALM => [0.3, 0.4, 0.6, 1.0],
            _ => [0.5, 0.7, 1.0, 1.0],
        }
    }

    pub fn spawn_y(&self) -> i32 {
        match *self {
            DimensionId::JUPITER | DimensionId::SATURN | DimensionId::NEPTUNE => 128,
            _ => 64,
        }
    }

    pub fn seed_offset(&self) -> u64 {
        self.0 as u64 * 1000
    }
}
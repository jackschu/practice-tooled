use crate::{armor_reducer::ArmorReducer, core::stat_at_level, load_champion::ChampionStats};

pub trait Target {
    fn get_vitality_data(&self) -> VitalityData;
}

#[derive(Default, Clone)]
pub struct VitalityData {
    pub base_armor: f64,
    pub bonus_armor: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub current_health: f64,
}

pub struct EffectData {
    pub ttl: f64,
    pub result: EffectResult,
}

pub enum EffectResult {
    ThreeHit { on_third_hit: Box<EffectResult> },
    Damage,
    ArmorReducer(ArmorReducer),
}

impl From<(&ChampionStats, u8)> for VitalityData {
    fn from(tuple: (&ChampionStats, u8)) -> VitalityData {
        let (stats, level) = tuple;
        let base_armor = stat_at_level(stats.armor, stats.armor_per_level, level);
        let magic_resist = stat_at_level(stats.magic_resist, stats.magic_resist_per_level, level);
        let max_health = stat_at_level(stats.health, stats.health_per_level, level);
        let target = VitalityData {
            base_armor,
            bonus_armor: 0.0,
            max_health,
            current_health: max_health,
            magic_resist,
        };

        return target;
    }
}

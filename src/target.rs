use crate::{
    armor_reducer::ArmorReducer,
    champions::champion::{self, Champion},
    core::stat_at_level,
    load_champion::ChampionStats,
};

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

pub struct ThreeHit {
    hit_count: u8,
    pub unique_name: String,
    pub on_third_hit: Box<EffectData>,
}

impl ThreeHit {
    pub fn upsert_to_champ(
        champion: &mut Champion,
        unique_name: String,
        effect: EffectData,
        ttl: f64,
    ) {
        let mut maybe_found: Option<ThreeHit> = None;
        if let Some(index) = champion.effects.iter().position(|e| match e.result {
            EffectResult::ThreeHit(_) => true,
            _ => false,
        }) {
            if let EffectResult::ThreeHit(out) = champion.effects.remove(index).result {
                maybe_found = Some(out);
            }
        }

        let mut found = maybe_found.unwrap_or(ThreeHit {
            hit_count: 0,
            //trigger_type,
            on_third_hit: Box::new(effect),
            unique_name,
        });
        found.hit_count += 1;
        if found.hit_count >= 3 {
            champion.add_effect(*found.on_third_hit);
        } else {
            champion.add_effect(EffectData {
                ttl,
                result: EffectResult::ThreeHit(found),
            });
        }
    }
}

pub enum EffectResult {
    ThreeHit(ThreeHit),
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

use std::{fmt, rc::Weak};

use crate::{
    armor_reducer::ArmorReducer,
    champions::champion::{CastingData, Champion, ChampionAbilites},
    core::stat_at_level,
    load_champion::ChampionStats,
    time_manager::TIME,
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

#[derive(Debug)]
pub struct EffectData {
    pub expiry: f64,
    pub unique_name: String,
    pub result: EffectResult,
}

#[derive(Debug)]
pub struct ThreeHit {
    hit_count: u8,
    pub on_third_hit: Box<EffectData>,
}

impl ThreeHit {
    pub fn upsert_to_champ(champion: &mut Champion, effect: EffectData, ttl: f64) {
        let mut maybe_found: Option<ThreeHit> = None;
        if let Some(index) = champion.effects.iter().position(|candidate| {
            candidate.unique_name == effect.unique_name
                && matches!(candidate.result, EffectResult::ThreeHit(_))
        }) {
            if let EffectResult::ThreeHit(out) = champion.effects.remove(index).result {
                maybe_found = Some(out);
            }
        }
        let three_hit_name = effect.unique_name.clone();

        let mut found = maybe_found.unwrap_or(ThreeHit {
            hit_count: 0,
            //trigger_type,
            on_third_hit: Box::new(effect),
        });
        found.hit_count += 1;
        if found.hit_count >= 3 {
            if let EffectResult::AbilityEffect {
                attacker,
                name,
                data,
            } = found.on_third_hit.result
            {
                Champion::execute_ability(attacker, name, champion, &data);
            } else {
                champion.upsert_effect(*found.on_third_hit);
            }
        } else {
            champion.add_effect(EffectData {
                unique_name: three_hit_name,
                expiry: TIME.with(|time| *time.borrow() + ttl),
                result: EffectResult::ThreeHit(found),
            });
        }
    }
}

pub enum EffectResult {
    ThreeHit(ThreeHit),
    ArmorReducer(ArmorReducer),
    AbilityEffect {
        attacker: Weak<Champion>,
        name: ChampionAbilites,
        data: CastingData,
    },
}

impl fmt::Debug for EffectResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ThreeHit(inside) => write!(f, "{:?}", inside),
            Self::ArmorReducer(inside) => write!(f, "{:?}", inside),
            Self::AbilityEffect {
                attacker: _,
                name,
                data,
            } => write!(f, "{:?} {:?}", name, data),
        }
    }
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

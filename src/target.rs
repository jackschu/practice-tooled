use std::{cell::RefCell, fmt, mem, rc::Weak};

use crate::{
    armor_reducer::ArmorReducer,
    champions::champion::{AbilityName, CastingData, Champion},
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
pub struct ThreeHitApplyInfo {
    pub ttl: f64,
    pub unique_name: String,
    pub result: Box<EffectResult>,
}

#[derive(Debug)]
pub struct ThreeHit {
    pub hit_count: u8,
    pub on_third_hit: ThreeHitApplyInfo,
}

impl PartialEq for EffectData {
    fn eq(&self, other: &Self) -> bool {
        self.unique_name == other.unique_name
            && mem::discriminant(&self.result) == mem::discriminant(&other.result)
    }
}

impl ThreeHit {
    pub fn upsert_to_champ(champion: &mut Champion, resulting_effect: ThreeHitApplyInfo, ttl: f64) {
        let three_hit_name = resulting_effect.unique_name.clone();
        let three_hit_effect = ThreeHit {
            hit_count: 0,
            on_third_hit: resulting_effect,
        };
        let three_hit_data = EffectData {
            unique_name: three_hit_name,
            expiry: TIME.with(|time| *time.borrow() + ttl),
            result: EffectResult::ThreeHit(three_hit_effect),
        };
        champion.upsert_effect(three_hit_data);
    }
}

pub enum EmpowerState {
    Cooldown,
    Active(AbilityEffect, f64),
}

#[derive(Clone)]
pub struct AbilityEffect {
    pub attacker: Weak<RefCell<Champion>>,
    pub name: AbilityName,
    pub data: CastingData,
}
pub enum EffectResult {
    ThreeHit(ThreeHit),
    ArmorReducer(ArmorReducer),
    EmpowerNextAttack(EmpowerState),
    AbilityEffect(AbilityEffect),
}

impl fmt::Debug for EffectResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ThreeHit(inside) => write!(f, "{:?}", inside),
            Self::ArmorReducer(inside) => write!(f, "{:?}", inside),
            Self::AbilityEffect(AbilityEffect {
                attacker: _,
                name,
                data,
            }) => write!(f, "AbilityEffect {:?} {:?}", name, data),
            Self::EmpowerNextAttack(inside) => write!(
                f,
                "EmpowerNextAttack {:?}",
                match inside {
                    EmpowerState::Active { .. } => "Active",
                    EmpowerState::Cooldown => "Cooldown",
                }
            ),
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

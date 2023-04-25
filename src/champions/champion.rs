use std::collections::HashMap;

use crate::{
    armor_reducer::ArmorReducer,
    attack::{BasicAttack, CritAdjuster, CritCalculation},
    core::{resist_damage, stat_at_level},
    load_champion::{load_champion_stats, ChampionStats},
    target::{EffectData, EffectResult, Target, VitalityData},
};

#[derive(Eq, Hash, PartialEq)]
pub enum ChampionAbilites {
    Q,
    W,
    E,
    R,
    AUTO,
}
pub struct Champion {
    pub stats: ChampionStats,
    pub level: u8,
    initial_armor: f64, // base armor before level ups
    pub current_health: f64,
    pub abilities: NamedClosures,
    pub crit_info: Option<(CritAdjuster, CritCalculation)>,
    pub effects: Vec<EffectData>,
}

#[derive(Default)]
pub struct CastingData {
    pub charge: f64,
    pub rank: u8,
}

impl CastingData {
    pub fn new(rank: u8) -> CastingData {
        CastingData {
            rank,
            ..Default::default()
        }
    }
}
pub struct NamedClosures {
    pub data: HashMap<ChampionAbilites, Box<dyn Fn(&mut Champion, &Champion, &CastingData) -> ()>>,
}

impl Champion {
    pub fn new_dummy_with_resist(armor: f64, magic_resist: f64) -> Champion {
        let health = 1000.0;
        let stats = ChampionStats {
            health,
            armor,
            magic_resist,
            ..Default::default()
        };

        return Champion {
            level: 1,
            stats,
            initial_armor: armor,
            current_health: health,
            abilities: NamedClosures {
                data: HashMap::new(),
            },
            crit_info: None,
            effects: Vec::new(),
        };
    }

    pub fn new_dummy() -> Champion {
        Champion::new_dummy_with_resist(0.0, 0.0)
    }

    pub fn new(level: u8, name: String, abilities: NamedClosures) -> Champion {
        let stats = load_champion_stats(name);
        let health = stat_at_level(stats.health, stats.health_per_level, level);
        let initial_armor = stats.armor;
        return Champion {
            level,
            stats,
            initial_armor,
            current_health: health,
            abilities,
            crit_info: None,
            effects: Vec::new(),
        };
    }

    pub fn add_effect(&mut self, effect: EffectData) {
        self.effects.push(effect)
    }

    pub fn execute_combo(
        &self,
        combo: Vec<(ChampionAbilites, CastingData)>,
        target: &mut Champion,
    ) {
        for (name, data) in combo {
            self.execute_ability(name, target, &data)
        }
    }

    pub fn execute_ability(
        &self,
        name: ChampionAbilites,
        target: &mut Champion,
        casting_data: &CastingData,
    ) {
        let func = self.abilities.data.get(&name).unwrap();
        func(target, self, casting_data);
    }

    pub fn get_base_armor(&self) -> f64 {
        return stat_at_level(self.initial_armor, self.stats.armor_per_level, self.level);
    }
    pub fn get_bonus_ad(&self) -> f64 {
        self.stats.bonus_attack_damage
    }

    pub fn get_bonus_armor(&self) -> f64 {
        return self.stats.armor - self.initial_armor;
    }

    pub fn get_max_health(&self) -> f64 {
        return stat_at_level(self.stats.health, self.stats.health_per_level, self.level);
    }

    pub fn get_missing_health(&self) -> f64 {
        self.get_max_health() - self.current_health
    }

    pub fn get_base_ad(&self) -> f64 {
        BasicAttack::from((&self.stats, self.level)).base_attack_damage
    }

    fn get_magic_resist(&self) -> f64 {
        return stat_at_level(
            self.stats.magic_resist,
            self.stats.magic_resist_per_level,
            self.level,
        );
    }

    pub fn full_heal(&mut self) {
        self.current_health = self.get_max_health()
    }

    pub fn receive_damage(&mut self, attacker: &Champion, damage: f64) {
        let mut armor_reducer: ArmorReducer = (&attacker.stats, attacker.level).into();
        self.effects
            .iter()
            .filter_map(|effect| match &effect.result {
                EffectResult::ArmorReducer(reducer) => Some(reducer),
                _ => None,
            })
            .for_each(|other_reducer| armor_reducer.add_armor_reducer(&other_reducer));

        let target_data = self.get_vitality_data();
        let effective_armor = armor_reducer.get_effective_armor(&target_data);
        let final_damage = resist_damage(damage, effective_armor);
        let health = &mut self.current_health;
        *health = *health - final_damage;
    }
}

impl Target for Champion {
    fn get_vitality_data(&self) -> VitalityData {
        return VitalityData {
            base_armor: self.get_base_armor(),
            bonus_armor: self.get_bonus_armor(),
            magic_resist: self.get_magic_resist(),
            max_health: self.get_max_health(),
            current_health: self.current_health,
        };
    }
}

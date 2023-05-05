use core::fmt;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    armor_reducer::ArmorReducer,
    attack::{BasicAttack, CritAdjuster, CritCalculation},
    core::{resist_damage, stat_at_level},
    item_effects::{OnHit, OnHitActivation, STATIC_ABILITIES},
    load_champion::{load_champion_stats, ChampionStats},
    target::{AbilityEffect, EffectData, EffectResult, EmpowerState, Target, VitalityData},
    time_manager::TIME,
};

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum AbilityName {
    Q,
    W,
    WPassive,
    E,
    R,
    AUTO,
    NIGHTSTALKER,
    SpellbladeSheen,
    SpellbladeEssenceReaver,
}
impl fmt::Display for AbilityName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct Champion {
    pub stats: ChampionStats,
    pub level: u8,
    initial_armor: f64, // base armor before level ups
    pub current_health: f64,
    pub abilities: NamedClosures,
    pub crit_info: Option<(CritAdjuster, CritCalculation)>,
    effects: Vec<EffectData>,
    pub on_hit_item_effects: Vec<OnHit>,
    pub ranks: [u8; 4],
}

#[derive(Default, Debug, Clone)]
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
    pub data:
        HashMap<AbilityName, Box<dyn Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) -> ()>>,
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
            on_hit_item_effects: Vec::new(),
            crit_info: None,
            effects: Vec::new(),
            ranks: [0, 0, 0, 0],
        };
    }

    pub fn new_dummy() -> Champion {
        Champion::new_dummy_with_resist(0.0, 0.0)
    }

    pub fn new(name: String, level: u8, ranks: [u8; 4], abilities: NamedClosures) -> Champion {
        let stats = load_champion_stats(name);
        let health = stat_at_level(stats.health, stats.health_per_level, level);
        let initial_armor = stats.armor;
        return Champion {
            level,
            stats,
            initial_armor,
            on_hit_item_effects: Vec::new(),
            current_health: health,
            abilities,
            crit_info: None,
            effects: Vec::new(),
            ranks,
        };
    }

    fn add_effect(&mut self, effect: EffectData) {
        self.effects.push(effect)
    }

    pub fn upsert_effect(&mut self, effect: EffectData) -> Option<()> {
        let other_expiry = effect.expiry;
        let mut to_add = effect;
        if let Some(index) = self
            .effects
            .iter()
            .position(|candidate| candidate == &to_add)
        {
            let maybe_expired = self.effects.remove(index);
            if TIME.with(|time| maybe_expired.expiry >= *time.borrow()) {
                to_add = self.bump_found_effect(maybe_expired)?;
                to_add.expiry = to_add.expiry.max(other_expiry);
            }
        }
        self.add_effect(to_add);
        Some(())
    }

    /**
     * Bumps the given effect, which entails modifying its run count, or increasing its expiry and / or executing it
     */
    fn bump_found_effect(&mut self, effect: EffectData) -> Option<EffectData> {
        match effect {
            EffectData {
                result: EffectResult::ThreeHit(mut three_hit_result),
                expiry,
                unique_name,
            } => {
                three_hit_result.hit_count += 1;
                if three_hit_result.hit_count >= 2 {
                    if let EffectResult::AbilityEffect(AbilityEffect {
                        attacker,
                        name,
                        data,
                    }) = *three_hit_result.on_third_hit.result
                    {
                        Champion::execute_ability(attacker, &name, self, &data);
                        return None;
                    } else {
                        self.upsert_effect(EffectData {
                            expiry: TIME
                                .with(|time| *time.borrow() + three_hit_result.on_third_hit.ttl),
                            unique_name: three_hit_result.on_third_hit.unique_name,
                            result: *three_hit_result.on_third_hit.result,
                        });
                        return None;
                    }
                } else {
                    return Some(EffectData {
                        expiry,
                        unique_name,
                        result: EffectResult::ThreeHit(three_hit_result),
                    });
                }
            }
            _ => Some(effect),
        }
    }

    pub fn execute_combo(
        attacker: Rc<RefCell<Self>>,
        combo: Vec<(AbilityName, CastingData)>,
        target: &mut Champion,
    ) {
        for (name, data) in combo {
            Self::execute_ability(Rc::downgrade(&attacker), &name, target, &data);
        }
    }

    fn process_on_hit_effects(
        attacker_ref: Weak<RefCell<Self>>,
        on_hit_effects: Vec<EffectData>,
    ) -> Option<()> {
        let attacker = attacker_ref.upgrade()?;

        on_hit_effects.into_iter().for_each(|effect| {
            attacker.borrow_mut().upsert_effect(effect);
        });

        return Some(());
    }

    fn process_on_auto_effects(
        attacker_ref: Weak<RefCell<Self>>,
        target: &mut Champion,
    ) -> Option<()> {
        let attacker = attacker_ref.upgrade()?;
        let to_cast: Vec<AbilityEffect> = attacker
            .borrow_mut()
            .valid_effects_mut()
            .filter_map(|effect| {
                let mut out: Option<AbilityEffect> = None;
                if let EffectResult::EmpowerNextAttack(result) = &mut effect.result {
                    if let EmpowerState::Active(ability, cd) = &result {
                        effect.expiry = TIME.with(|time| *time.borrow() + cd);
                        out = Some(ability.clone());
                    }
                    effect.result = EffectResult::EmpowerNextAttack(EmpowerState::Cooldown);
                }
                return out;
            })
            .collect();

        to_cast.iter().for_each(|ability| {
            Champion::execute_ability(
                Weak::clone(&ability.attacker),
                &ability.name,
                target,
                &ability.data,
            );
        });
        return Some(());
    }

    pub fn execute_ability(
        attacker_ref: Weak<RefCell<Self>>,
        name: &AbilityName,
        target: &mut Champion,
        casting_data: &CastingData,
    ) -> Option<f64> {
        let initial_health = target.current_health;
        let attacker = attacker_ref.upgrade()?;
        match name {
            AbilityName::AUTO => {
                let on_auto_effects: Vec<EffectData> = attacker
                    .borrow()
                    .on_hit_item_effects
                    .iter()
                    .filter(|effect| matches!(effect.mode, OnHitActivation::Auto))
                    .map(|on_hit| (on_hit, Weak::clone(&attacker_ref)).into())
                    .collect();
                Champion::process_on_hit_effects(Weak::clone(&attacker_ref), on_auto_effects);
                Champion::process_on_auto_effects(attacker_ref, target);
            }
            AbilityName::Q | AbilityName::W | AbilityName::E | AbilityName::R => {
                let on_auto_effects: Vec<EffectData> = attacker
                    .borrow()
                    .on_hit_item_effects
                    .iter()
                    .filter(|effect| matches!(effect.mode, OnHitActivation::ActiveSpell))
                    .map(|on_hit| (on_hit, Weak::clone(&attacker_ref)).into())
                    .collect();
                Champion::process_on_hit_effects(Weak::clone(&attacker_ref), on_auto_effects);
            }
            _ => {}
        }

        let binding = attacker.borrow();
        let maybe_func = binding.abilities.data.get(&name);
        if let Some(func) = maybe_func {
            func(target, Rc::clone(&attacker), casting_data);
        } else {
            STATIC_ABILITIES.with(|item_abilities| {
                let func = item_abilities.get(&name).unwrap();
                func(target, Rc::clone(&attacker), casting_data);
            })
        }
        let final_health = target.current_health;
        return Some(initial_health - final_health);
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
        self.valid_effects()
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

    pub fn valid_effects(&self) -> impl Iterator<Item = &EffectData> {
        self.effects
            .iter()
            .filter(|effect| TIME.with(|time| effect.expiry >= *time.borrow()))
    }
    pub fn valid_effects_mut(&mut self) -> impl Iterator<Item = &mut EffectData> {
        self.effects
            .iter_mut()
            .filter(|effect| TIME.with(|time| effect.expiry >= *time.borrow()))
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

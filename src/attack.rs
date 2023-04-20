use crate::{armor_reducer::ArmorReducer, load_champion::ChampionStats, target::Target};

use super::core;

pub struct CritAdjuster {
    pub critical_strike_chance: f64,
    pub bonus_critical_damage: f64,
}

impl CritAdjuster {
    pub fn get_multipler(&self, crit_calc: &CritCalculation) -> f64 {
        return match crit_calc {
            CritCalculation::NoCrit => 1.0,
            CritCalculation::DidCrit => 1.75 + self.bonus_critical_damage,
            CritCalculation::AverageOutcome => {
                1.0 + (self.critical_strike_chance * (0.75f64 + self.bonus_critical_damage))
            }
        };
    }
}

#[derive(Default, Clone)]
pub struct BasicAttack {
    pub base_attack_damage: f64,
    pub bonus_attack_damage: f64,
}

#[derive(Default, Clone)]
pub struct AttackSpeed {
    pub base: f64,
    pub bonus: f64,
}

impl AttackSpeed {
    pub fn get_attacks_per_second(&self) -> f64 {
        return self.base * (1.0 + self.bonus / 100.0);
    }
}

impl BasicAttack {
    pub fn get_total_attack_damage(&self) -> f64 {
        return self.base_attack_damage + self.bonus_attack_damage;
    }
    pub fn get_damage_to_target(
        &self,
        target: &Target,
        crit_adjuster: &Option<(&CritAdjuster, CritCalculation)>,
        armor_reducer: Option<&ArmorReducer>,
    ) -> f64 {
        let damage = self.get_total_attack_damage();

        let effective_armor = match armor_reducer {
            Some(reducer) => reducer.get_effective_armor(&target),
            None => target.base_armor + target.bonus_armor,
        };

        let adjusted_crit_multipier = match crit_adjuster {
            Some((adjuster, crit_calc)) => adjuster.get_multipler(crit_calc),
            None => 1.0,
        };
        return core::resist_damage(damage, effective_armor) * adjusted_crit_multipier;
    }

    pub fn new(base_attack_damage: f64, bonus_attack_damage: f64) -> Self {
        Self {
            base_attack_damage,
            bonus_attack_damage,
        }
    }
}

impl From<(&ChampionStats, u8)> for BasicAttack {
    fn from(tuple: (&ChampionStats, u8)) -> BasicAttack {
        let (stats, level) = tuple;
        let attack_damage = core::stat_at_level(
            stats.base_attack_damage,
            stats.attack_damage_per_level,
            level,
        ) + stats.bonus_attack_damage;
        let attack = BasicAttack {
            base_attack_damage: attack_damage,
            ..Default::default()
        };

        return attack;
    }
}

impl From<(&ChampionStats, u8)> for AttackSpeed {
    fn from(tuple: (&ChampionStats, u8)) -> AttackSpeed {
        let (stats, level) = tuple;
        let bonus_speed = core::stat_at_level(0.0, stats.attack_speed_per_level, level)
            + stats.bonus_attack_speed;
        return AttackSpeed {
            base: stats.attack_speed,
            bonus: bonus_speed,
        };
    }
}

pub enum CritCalculation {
    DidCrit,
    NoCrit,
    AverageOutcome,
}

pub fn get_dps(
    attack_speed: &AttackSpeed,
    attack: &BasicAttack,
    target: &Target,
    armor_reducer: Option<&ArmorReducer>,
    crit_adjuster: Option<&CritAdjuster>,
) -> f64 {
    const CRIT_CALC: CritCalculation = CritCalculation::AverageOutcome;
    let crit_tuple = crit_adjuster.map(|v| (v, CRIT_CALC));
    let damage = attack.get_damage_to_target(target, &crit_tuple, armor_reducer);
    return damage * attack_speed.get_attacks_per_second();
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    mod basic_attack {
        use super::*;
        #[test]
        fn typical_attack() {
            let attack = BasicAttack {
                base_attack_damage: 24.0,
                bonus_attack_damage: 50.0,
                ..Default::default()
            };

            let target = Target {
                base_armor: 20.0,
                ..Default::default()
            };

            let damage = attack.get_damage_to_target(&target, &None, None);
            assert_eq!(62, damage.round() as u32)
        }
    }

    #[test]
    fn test_attack_to_attack_per_second() {
        let speed = AttackSpeed {
            base: 0.651,
            bonus: 102.9228,
        };
        let per_second = speed.get_attacks_per_second();

        assert_relative_eq!(1.321027428, per_second);
    }
}

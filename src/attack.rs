use crate::core::lethality_to_pen;

use super::core;

#[derive(Default, Clone)]
pub struct ArmorReducer {
    pub flat_armor_reduction: f64,
    pub percent_armor_reduction: f64,

    pub flat_armor_pen: f64, // effective lethality
    pub percent_armor_pen: f64,
    pub percent_bonus_armor_pen: f64,
}

impl ArmorReducer {
    pub fn apply_armor_reduction(&self, target: &mut Target) {
        let total_armor = target.base_armor + target.bonus_armor;
        let base_ratio = if total_armor != 0.0 {
            target.base_armor / total_armor
        } else {
            0.5
        };
        let bonus_ratio = 1.0 - base_ratio;
        target.base_armor -= self.flat_armor_reduction * base_ratio;
        target.bonus_armor -= self.flat_armor_reduction * bonus_ratio;

        if target.base_armor > 0.0 {
            target.base_armor *= 1.0 - self.percent_armor_reduction / 100.0;
        }
        if target.bonus_armor > 0.0 {
            target.bonus_armor *= 1.0 - self.percent_armor_reduction / 100.0;
        }
    }

    pub fn get_effective_armor(&self, original_target: &Target) -> f64 {
        let mut target = original_target.clone();
        self.apply_armor_reduction(&mut target);
        let mut effective_armor =
            target.bonus_armor * (1.0 - self.percent_bonus_armor_pen / 100.0) + target.base_armor;
        effective_armor *= 1.0 - self.percent_armor_pen / 100.0;

        // lethality can't reduce below 0
        if effective_armor < 0.0 {
            return effective_armor;
        }
        let candidate = effective_armor - self.flat_armor_pen;
        if candidate < 0.0 {
            return 0.0;
        } else {
            return candidate;
        }
    }

    pub fn set_from_lethality(&mut self, lethality: f64, level: u8) {
        self.flat_armor_pen = lethality_to_pen(lethality, level);
    }
}

#[derive(Default, Clone)]
pub struct BasicAttack {
    pub base_attack_damage: f64,
    pub bonus_attack_damage: f64,
    pub critical_strike_chance: f64,
    pub bonus_critical_damage: f64,
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
        crit_calc: CritCalculation,
        armor_reducer: Option<&ArmorReducer>,
    ) -> f64 {
        let damage = self.get_total_attack_damage();

        let effective_armor = match armor_reducer {
            Some(reducer) => reducer.get_effective_armor(&target),
            None => target.base_armor + target.bonus_armor,
        };

        let adjusted_crit_multipier = match crit_calc {
            CritCalculation::NoCrit => 1.0,
            CritCalculation::DidCrit => 1.75 + self.bonus_critical_damage,
            CritCalculation::AverageOutcome => {
                1.0 + (self.critical_strike_chance * (0.75f64 + self.bonus_critical_damage))
            }
        };
        return core::resist_damage(damage, effective_armor) * adjusted_crit_multipier;
    }

    pub fn new(base_attack_damage: f64, bonus_attack_damage: f64) -> Self {
        Self {
            base_attack_damage,
            bonus_attack_damage,
            critical_strike_chance: 0.0,
            bonus_critical_damage: 0.0,
        }
    }
}

#[derive(Default, Clone)]
pub struct Target {
    base_armor: f64,
    bonus_armor: f64,
    magic_resist: f64,
    max_health: f64,
    current_health: f64,
}

#[derive(Default)]
pub struct TargetData {
    pub base_armor: f64,
    pub bonus_armor: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub current_health: f64,
}

impl Target {
    pub fn new(input: TargetData) -> Self {
        Self {
            base_armor: input.base_armor,
            bonus_armor: input.bonus_armor,
            magic_resist: input.magic_resist,
            max_health: input.max_health,
            current_health: input.current_health,
        }
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
) -> f64 {
    const CRIT_CALC: CritCalculation = CritCalculation::AverageOutcome;
    let damage = attack.get_damage_to_target(target, CRIT_CALC, armor_reducer);
    return damage * attack_speed.get_attacks_per_second();
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rstest::rstest;

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

            let damage =
                attack.get_damage_to_target(&target, CritCalculation::AverageOutcome, None);
            assert_eq!(62, damage.round() as u32)
        }
    }

    #[rstest]
    // flat reduction wiki example
    #[case(20.0, 40.0, ArmorReducer{ flat_armor_reduction: 30.0, ..Default::default() } , 30.0)]
    // percent reduction wiki example
    #[case(20.0, 40.0, ArmorReducer{ percent_armor_reduction: 30.0, ..Default::default() } , 42.0)]
    // percent pen wiki example
    #[case(20.0, 40.0, ArmorReducer{ percent_armor_pen: 30.0, ..Default::default() } , 42.0)]
    // percent bonus pen wiki example
    #[case(20.0, 40.0, ArmorReducer{
		percent_armor_pen: 10.0,
		percent_bonus_armor_pen: 30.0,
		..Default::default() } , 43.2
	)]
    // high level wiki example
    #[case(100.0, 200.0, ArmorReducer{
		percent_bonus_armor_pen: 45.0,
		flat_armor_pen: 10.0,
		flat_armor_reduction: 30.0,
		percent_armor_reduction: 30.0,
		..Default::default()
	} , 122.3)]
    // high level wiki example
    #[case(18.0, 0.0, ArmorReducer{
		percent_bonus_armor_pen: 45.0,
		flat_armor_pen: 10.0,
		flat_armor_reduction: 30.0,
		percent_armor_reduction: 30.0,
		..Default::default()
	} , -12.0)]
    fn effective_armor(
        #[case] base_armor: f64,
        #[case] bonus_armor: f64,
        #[case] reducer: ArmorReducer,
        #[case] expected_armor: f64,
    ) {
        let target = Target::new(TargetData {
            base_armor,
            bonus_armor,
            ..Default::default()
        });

        assert_relative_eq!(expected_armor, reducer.get_effective_armor(&target));
    }

    #[rstest]
    fn test_attack_to_attack_per_second() {
        let speed = AttackSpeed {
            base: 0.651,
            bonus: 102.9228,
        };
        let per_second = speed.get_attacks_per_second();

        assert_relative_eq!(1.321027428, per_second);
    }
}

use super::core;

#[derive(Default)]
pub struct BasicAttack {
    attack_damage: f64,
    critical_strike_chance: f64,
    bonus_critical_damage: f64,

    flat_armor_reduction: f64,
    percent_armor_reduction: f64,

    flat_armor_pen: f64, // effective lethality
    percent_armor_pen: f64,
    percent_bonus_armor_pen: f64,
}

impl BasicAttack {
    pub fn new(attack_damage: f64) -> Self {
        Self {
            attack_damage,
            critical_strike_chance: 0.0,
            bonus_critical_damage: 0.0,

            flat_armor_reduction: 0.0,
            percent_armor_reduction: 0.0,

            flat_armor_pen: 0.0, // effective lethality
            percent_armor_pen: 0.0,
            percent_bonus_armor_pen: 0.0,
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

fn apply_armor_reduction(attack: &BasicAttack, target: &mut Target) {
    let base_ratio = target.base_armor / (target.base_armor + target.bonus_armor);
    let bonus_ratio = 1.0 - base_ratio;
    target.base_armor -= attack.flat_armor_reduction * base_ratio;
    target.bonus_armor -= attack.flat_armor_reduction * bonus_ratio;
	if target.base_armor > 0.0 {
		target.base_armor *= 1.0 - attack.percent_armor_reduction / 100.0;
	}
	if target.bonus_armor > 0.0 {
		target.bonus_armor *= 1.0 - attack.percent_armor_reduction / 100.0;
	}
}

fn get_effetive_armor(attack: &BasicAttack, original_target: &Target) -> f64 {
    let mut target = original_target.clone();
    apply_armor_reduction(attack, &mut target);
    let mut effective_armor =
        target.bonus_armor * (1.0 - attack.percent_bonus_armor_pen / 100.0) + target.base_armor;
    effective_armor *= 1.0 - attack.percent_armor_pen / 100.0;

    // lethality can't reduce below 0
    if effective_armor < 0.0 {
        return effective_armor;
    }
    let candidate = effective_armor - attack.flat_armor_pen;
    if candidate < 0.0 {
        return 0.0;
    } else {
        return candidate;
    }
}

pub fn get_basic_attack_damage(
    attack: BasicAttack,
    target: Target,
    crit_calc: CritCalculation,
) -> f64 {
    let damage = attack.attack_damage;
    let effective_armor = get_effetive_armor(&attack, &target);

    let adjusted_crit_multipier = match crit_calc {
        CritCalculation::DidCrit => 1.0,
        CritCalculation::NoCrit => 1.75 + attack.bonus_critical_damage,
        CritCalculation::AverageOutcome => {
            1.0 + (attack.critical_strike_chance * (0.75f64 + attack.bonus_critical_damage))
        }
    };
    return core::resist_damage(damage, effective_armor) * adjusted_crit_multipier;
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
                attack_damage: 74.0,
                ..Default::default()
            };

            let target = Target {
                base_armor: 20.0,
                ..Default::default()
            };

            let damage = get_basic_attack_damage(attack, target, CritCalculation::AverageOutcome);
            assert_eq!(62, damage.round() as u32)
        }
    }

    #[rstest]
    // flat reduction wiki example
    #[case(20.0, 40.0, BasicAttack{ flat_armor_reduction: 30.0, ..Default::default() } , 30.0)]
    // percent reduction wiki example
    #[case(20.0, 40.0, BasicAttack{ percent_armor_reduction: 30.0, ..Default::default() } , 42.0)]
    // percent pen wiki example
    #[case(20.0, 40.0, BasicAttack{ percent_armor_pen: 30.0, ..Default::default() } , 42.0)]
    // percent bonus pen wiki example
    #[case(20.0, 40.0, BasicAttack{
		percent_armor_pen: 10.0,
		percent_bonus_armor_pen: 30.0,
		..Default::default() } , 43.2
	)]
    // high level wiki example
    #[case(100.0, 200.0, BasicAttack{
		percent_bonus_armor_pen: 45.0,
		flat_armor_pen: 10.0,
		flat_armor_reduction: 30.0,
		percent_armor_reduction: 30.0,
		..Default::default()
	} , 122.3)]
    // high level wiki example
    #[case(18.0, 0.0, BasicAttack{
		percent_bonus_armor_pen: 45.0,
		flat_armor_pen: 10.0,
		flat_armor_reduction: 30.0,
		percent_armor_reduction: 30.0,
		..Default::default()
	} , -12.0)]
    fn effective_armor(
        #[case] base_armor: f64,
        #[case] bonus_armor: f64,
        #[case] attack: BasicAttack,
        #[case] expected_armor: f64,
    ) {
        let target = Target {
            base_armor,
            bonus_armor,
            ..Default::default()
        };

        assert_relative_eq!(expected_armor, get_effetive_armor(&attack, &target));
    }
}

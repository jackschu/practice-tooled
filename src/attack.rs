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
            critical_strike_chance: 0f64,
            bonus_critical_damage: 0f64,

            flat_armor_reduction: 0f64,
            percent_armor_reduction: 0f64,

            flat_armor_pen: 0f64, // effective lethality
            percent_armor_pen: 0f64,
            percent_bonus_armor_pen: 0f64,
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
    let bonus_ratio = 1f64 - base_ratio;
    target.base_armor -= attack.flat_armor_reduction * base_ratio;
    target.bonus_armor -= attack.flat_armor_reduction * bonus_ratio;
    target.base_armor *= 1f64 - attack.percent_armor_reduction / 100f64;
    target.bonus_armor *= 1f64 - attack.percent_armor_reduction / 100f64;
}

fn get_effetive_armor(attack: &BasicAttack, original_target: &Target) -> f64 {
    let mut target = original_target.clone();
    apply_armor_reduction(attack, &mut target);
    let mut effective_armor =
        target.bonus_armor * (1f64 - attack.percent_bonus_armor_pen / 100f64) + target.base_armor;
    effective_armor *= 1f64 - attack.percent_armor_pen / 100f64;

    // lethality can't reduce below 0
    if effective_armor < 0f64 {
        return effective_armor;
    }
    let candidate = effective_armor - attack.flat_armor_pen;
    if candidate < 0f64 {
        return 0f64;
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
        CritCalculation::DidCrit => 1f64,
        CritCalculation::NoCrit => 1.75f64 + attack.bonus_critical_damage,
        CritCalculation::AverageOutcome => {
            1f64 + (attack.critical_strike_chance * (0.75f64 + attack.bonus_critical_damage))
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
                attack_damage: 74f64,
                ..Default::default()
            };

            let target = Target {
                base_armor: 20f64,
                ..Default::default()
            };

            let damage = get_basic_attack_damage(attack, target, CritCalculation::AverageOutcome);
            assert_eq!(62, damage.round() as u32)
        }
    }

    #[rstest]
    #[case(20f64, 40f64, BasicAttack{ flat_armor_reduction: 30f64, ..Default::default() } , 30f64)]
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

        let attack = BasicAttack {
            flat_armor_reduction: 30f64,
            ..Default::default()
        };

        assert_relative_eq!(expected_armor, get_effetive_armor(&attack, &target));
    }
}

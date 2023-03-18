use super::core;


#[derive(Default)]
pub struct BasicAttack {
    attack_damage: f64,
    critical_strike_chance: f64,
    bonus_critical_damage: f64,

    flat_armor_reduction: f64,
    percent_armor_reduction: f64,

    flat_armor_pen: f64,
    percent_armor_pen: f64,
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
        }
    }
}

#[derive(Default)]
pub struct Target {
    armor: f64,
    magic_resist: f64,
    max_health: f64,
    current_health: f64,
}

#[derive(Default)]
pub struct TargetData {
    pub armor: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub current_health: f64,
}

impl Target {
    pub fn new(input: TargetData) -> Self {
        Self {
			armor: input.armor,
			magic_resist: input.magic_resist,
			max_health: input.max_health,
			current_health: input.current_health,
        }
    }
}

pub enum CritCalculation {
	DidCrit,
	NoCrit,
	AverageOutcome
}

pub fn get_basic_attack_damage(attack: BasicAttack, target: Target, crit_calc: CritCalculation) -> f64 {
    let damage = attack.attack_damage;
    let mut effective_armor = target.armor;
    effective_armor -= attack.flat_armor_reduction;
    effective_armor *= 1f64 - attack.percent_armor_reduction / 100f64;
    effective_armor *= 1f64 - attack.percent_armor_pen / 100f64;
    effective_armor -= attack.flat_armor_pen;


	let adjusted_crit_multipier = match crit_calc {
		CritCalculation::DidCrit => 1f64,
		CritCalculation::NoCrit => 1.75f64 + attack.bonus_critical_damage,
		CritCalculation::AverageOutcome => 1f64 + (attack.critical_strike_chance * (0.75f64  + attack.bonus_critical_damage)),
	};
    return core::resist_damage(damage, effective_armor) * adjusted_crit_multipier;
}

#[cfg(test)]
mod tests {
    use super::*;
    mod basic_attack {
        use super::*;
        #[test]
        fn typical_attack() {
            let attack = BasicAttack{
				attack_damage: 74f64,
				..Default::default()
			};

            let target = Target {
                armor: 20f64,
                ..Default::default()
            };

            let damage = get_basic_attack_damage(attack, target, CritCalculation::AverageOutcome);
            assert_eq!(62, damage.round() as u32)
        }
    }
}

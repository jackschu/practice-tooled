use practice_tooled::attack;

#[cfg(test)]
mod tests {
    use super::*;
    use attack::*;
    use practice_tooled::core::*;
    use rstest::rstest;

    #[rstest]
	// values sampled from game
    #[case(0.0, 0.0, 2, 10.0, BasicAttack{
		attack_damage: 115.0,
		percent_armor_pen: 18.0,
		..Default::default()
	} , 115)]
    #[case(0.0, 20.0, 2, 10.0, BasicAttack{
		attack_damage: 115.0,
		percent_armor_pen: 18.0,
		..Default::default()
	} , 105)]
    fn test_resist_damage(
        #[case] base_armor: f64,
        #[case] bonus_armor: f64,

        #[case] level: u32,
        #[case] lethality: f64,

        #[case] initial_attack: BasicAttack,
        #[case] expected_damage: u32,
    ) {
        let target = Target::new(TargetData {
            base_armor,
            bonus_armor,
            ..Default::default()
        });

        let mut attack = initial_attack.clone();
        attack.flat_armor_pen = lethality_to_pen(lethality, level);
        let observed_damage = get_basic_attack_damage(attack, target, CritCalculation::NoCrit);
        assert_eq!(expected_damage, observed_damage.round() as u32);
    }
}

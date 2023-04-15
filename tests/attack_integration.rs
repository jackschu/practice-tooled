use practice_tooled::attack;

#[cfg(test)]
mod tests {
    use super::*;
    use attack::*;
    use rstest::rstest;

    #[rstest]
    // values sampled from game
    #[case(0.0, 0.0, 2, 10.0, BasicAttack{
        base_attack_damage: 115.0,
        ..Default::default()
    }, ArmorReducer{
        percent_armor_pen: 18.0,
        ..Default::default()
    } , 115)]
    #[case(0.0, 20.0, 2, 10.0, BasicAttack{
        base_attack_damage: 115.0,
        ..Default::default()
    } , ArmorReducer{
        percent_armor_pen: 18.0,
        ..Default::default()
    }, 105)]
    fn test_resist_damage(
        #[case] base_armor: f64,
        #[case] bonus_armor: f64,

        #[case] level: u8,
        #[case] lethality: f64,

        #[case] initial_attack: BasicAttack,

        #[case] mut reducer: ArmorReducer,

        #[case] expected_damage: u32,
    ) {
        let target = Target::new(TargetData {
            base_armor,
            bonus_armor,
            ..Default::default()
        });

        let attack = initial_attack.clone();
        reducer.set_from_lethality(lethality, level);
        let observed_damage =
            attack.get_damage_to_target(&target, CritCalculation::NoCrit, Some(&reducer));
        assert_eq!(expected_damage, observed_damage.round() as u32);
    }
}

use practice_tooled::attack;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use approx::assert_relative_eq;
    use attack::*;
    use practice_tooled::{
        armor_reducer::ArmorReducer,
        champions::{
            champion::{AbilityName, CastingData, Champion},
            Vi,
        },
        load_wiki_item::apply_item_to_champ,
        target::VitalityData,
        time_manager::TIME,
    };
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
        let target = VitalityData {
            base_armor,
            bonus_armor,
            ..Default::default()
        };

        let attack = initial_attack.clone();
        reducer.set_from_lethality(lethality, level);
        let observed_damage = attack.get_damage_to_target(&target, &None, Some(&reducer));
        assert_eq!(expected_damage, observed_damage.round() as u32);
    }

    #[rstest]
    fn test_nighstalker() {
        let level = 6;
        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let vi = Rc::new(RefCell::new(Champion::new(
            Vi::NAME.to_string(),
            level,
            [0, 0, 0, 0],
            vi_closures,
        )));

        apply_item_to_champ("Duskblade of Draktharr", &mut vi.borrow_mut());

        let target = &mut Champion::new_dummy();
        let first_proc = Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::AUTO,
            target,
            &CastingData {
                ..Default::default()
            },
        )
        .unwrap();
        TIME.with(|time| *time.borrow_mut() += 20.0);
        let second_proc = Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::AUTO,
            target,
            &CastingData {
                ..Default::default()
            },
        )
        .unwrap();

        //first and second duskblade procs do equal damage (due to delay)
        assert_relative_eq!(first_proc, second_proc);

        TIME.with(|time| *time.borrow_mut() += 5.0);
        let third_auto = Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::AUTO,
            target,
            &CastingData {
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            third_auto < second_proc,
            "third auto {:2} shouldnt be a duskblade proc and do less than second {:2}",
            third_auto,
            second_proc
        );
    }

    #[rstest]
    #[case(5.0, 0.0, (true, false))]
    #[case(5.0, 2.0, (true, true))]
    #[case(15.0, 0.0, (false, true))]
    fn test_sheen(
        #[case] ability_delay: f64,
        #[case] auto_delay: f64,
        #[case] expect_empowered: (bool, bool),
    ) {
        let level = 6;
        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let vi = Rc::new(RefCell::new(Champion::new(
            Vi::NAME.to_string(),
            level,
            [0, 0, 0, 0],
            vi_closures,
        )));

        apply_item_to_champ("Sheen", &mut vi.borrow_mut());

        let target = &mut Champion::new_dummy();
        let base_auto = Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::AUTO,
            target,
            &CastingData {
                ..Default::default()
            },
        )
        .unwrap();
        Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::R,
            target,
            &CastingData {
                ..Default::default()
            },
        );
        TIME.with(|time| *time.borrow_mut() += ability_delay);

        let empowered_auto = Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::AUTO,
            target,
            &CastingData {
                ..Default::default()
            },
        )
        .unwrap();
        println!("{base_auto} {empowered_auto}");

        TIME.with(|time| *time.borrow_mut() += auto_delay);
        Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::R,
            target,
            &CastingData {
                ..Default::default()
            },
        );
        let second_base_auto = Champion::execute_ability(
            Rc::downgrade(&vi),
            &AbilityName::AUTO,
            target,
            &CastingData {
                ..Default::default()
            },
        )
        .unwrap();
        let (first, second) = expect_empowered;
        if first {
            assert!(base_auto < empowered_auto);
        } else {
            assert_relative_eq!(base_auto, empowered_auto);
        }

        if second {
            assert!(
                base_auto < second_base_auto,
                "l {}, r {}",
                base_auto,
                second_base_auto
            );
        } else {
            assert_relative_eq!(base_auto, second_base_auto);
        }
    }
}

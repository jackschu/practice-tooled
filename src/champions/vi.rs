use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    armor_reducer::ArmorReducer,
    attack::BasicAttack,
    target::{AbilityEffect, EffectResult, ThreeHit, ThreeHitApplyInfo, VitalityData},
};

use super::champion::{CastingData, Champion, ChampionAbilites, NamedClosures};

pub struct Vi {
    q_data: AbiltyDamageInfo,
    w_data: AbiltyDamageInfo,
    e_data: AbiltyDamageInfo,
    r_data: AbiltyDamageInfo,
}

#[derive(Default, Clone, Copy)]
pub struct AbiltyDamageInfo {
    pub base_damages: [f64; 5],
    pub target_max_health_ratio: [f64; 5], // stored as percent (0-100)
    pub ad_ratio: f64,
    pub bonus_ad_ratio: f64,
}

impl AbiltyDamageInfo {
    pub fn to_damage_amount(&self, rank: u8, base: f64, bonus: f64) -> f64 {
        return self.base_damages[rank as usize]
            + 0.01 * self.ad_ratio * (base + bonus)
            + 0.01 * self.bonus_ad_ratio * bonus;
    }
}

impl Vi {
    pub const NAME: &str = "Vi";

    // as of 13.7
    #[allow(dead_code)]
    const Q_CD: [f64; 5] = [12.0, 10.5, 9.0, 7.5, 6.0];
    const Q_DAMAGE: [f64; 5] = [45.0, 70.0, 95.0, 120.0, 145.0];
    const Q_MAX_DAMAGE_CHARGE: f64 = 1.25;

    const W_HP_SCALING: [f64; 5] = [4.0, 5.5, 7.0, 8.5, 10.0];

    const E_DAMAGE: [f64; 5] = [0.0, 15.0, 30.0, 45.0, 60.0];
    const R_DAMAGE: [f64; 5] = [150.0, 325.0, 350.0, 0.0, 0.0];

    pub fn new() -> Vi {
        Vi {
            w_data: AbiltyDamageInfo {
                bonus_ad_ratio: 1.0 / 35.0, // percent ad
                target_max_health_ratio: Vi::W_HP_SCALING,
                ..Default::default()
            },
            e_data: AbiltyDamageInfo {
                base_damages: Vi::E_DAMAGE,
                ad_ratio: 120.0,
                ..Default::default()
            },
            r_data: AbiltyDamageInfo {
                base_damages: Vi::R_DAMAGE,
                bonus_ad_ratio: 110.0,
                ..Default::default()
            },

            q_data: AbiltyDamageInfo {
                base_damages: Vi::Q_DAMAGE,
                bonus_ad_ratio: 80.0,
                ..Default::default()
            },
        }
    }

    pub fn get_name_closures(&mut self) -> NamedClosures {
        let mut map: HashMap<
            ChampionAbilites,
            Box<dyn Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) -> ()>,
        > = HashMap::new();
        map.entry(ChampionAbilites::Q)
            .or_insert(Box::new(Vi::ability_q(self.q_data)));
        map.entry(ChampionAbilites::W)
            .or_insert(Box::new(Vi::ability_w(self.w_data)));
        map.entry(ChampionAbilites::E)
            .or_insert(Box::new(Vi::ability_e(self.e_data)));
        map.entry(ChampionAbilites::R)
            .or_insert(Box::new(Vi::ability_r(self.r_data)));

        map.entry(ChampionAbilites::AUTO)
            .or_insert(Box::new(Vi::auto_attack()));

        return NamedClosures { data: map };
    }

    pub fn ability_q(
        q_data: AbiltyDamageInfo,
    ) -> impl Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) {
        return move |target: &mut Champion,
                     attacker: Rc<RefCell<Champion>>,
                     casting_data: &CastingData| {
            const MAX_SCALE: f64 = 1.0;
            let rank = casting_data.rank;
            let percent_damage = MAX_SCALE.min(casting_data.charge * 0.10 / 0.125) + 1.0;
            let base_ad = attacker.borrow().get_base_ad();
            let bonus_ad = attacker.borrow().get_bonus_ad();

            let mut raw_damage = q_data.to_damage_amount(rank, base_ad, bonus_ad);
            raw_damage *= percent_damage;
            target.receive_damage(&attacker.borrow(), raw_damage);
            Vi::apply_w_effect(target, attacker);
        };
    }

    pub fn apply_w_effect(target: &mut Champion, attacker: Rc<RefCell<Champion>>) {
        ThreeHit::upsert_to_champ(
            target,
            ThreeHitApplyInfo {
                unique_name: "Denting Blows Damage".to_string(),
                result: Box::new(EffectResult::AbilityEffect(AbilityEffect {
                    attacker: Rc::downgrade(&attacker),
                    name: ChampionAbilites::W,
                    data: CastingData {
                        rank: attacker.borrow().ranks[1],
                        ..Default::default()
                    },
                })),
                ttl: 0.0,
            },
            4.0,
        );
        ThreeHit::upsert_to_champ(
            target,
            ThreeHitApplyInfo {
                unique_name: "Denting Blows Armor".to_string(),
                result: Box::new(EffectResult::ArmorReducer(ArmorReducer {
                    percent_armor_reduction: 20.0,
                    ..Default::default()
                })),
                ttl: 4.0,
            },
            4.0,
        )
    }

    pub fn ability_w(
        w_data: AbiltyDamageInfo,
    ) -> impl Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) {
        return move |target: &mut Champion,
                     attacker: Rc<RefCell<Champion>>,
                     casting_data: &CastingData| {
            let bonus_ad = attacker.borrow().get_bonus_ad();
            let rank = casting_data.rank;
            let percent_health_dmg = 0.01 * w_data.target_max_health_ratio[rank as usize]
                + 0.01 * w_data.bonus_ad_ratio * bonus_ad;
            let raw_damage = percent_health_dmg * target.get_max_health();
            target.receive_damage(&attacker.borrow(), raw_damage);
        };
    }

    pub fn ability_e(
        e_data: AbiltyDamageInfo,
    ) -> impl Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) {
        return move |target: &mut Champion,
                     attacker: Rc<RefCell<Champion>>,
                     casting_data: &CastingData| {
            let rank = casting_data.rank;
            let bonus_ad = attacker.borrow().get_bonus_ad();
            let base_ad = attacker.borrow().get_base_ad();
            let e_dmg = e_data.to_damage_amount(rank, base_ad, bonus_ad);
            let attack = BasicAttack::new(e_dmg, 0.0);
            let raw_damage = attack.get_damage_to_target(
                &VitalityData::default(),
                &attacker.borrow().crit_info,
                None,
            );
            target.receive_damage(&attacker.borrow(), raw_damage);
            Vi::apply_w_effect(target, attacker);
        };
    }

    pub fn ability_r(
        r_data: AbiltyDamageInfo,
    ) -> impl Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) {
        return move |target: &mut Champion,
                     attacker: Rc<RefCell<Champion>>,
                     casting_data: &CastingData| {
            let rank = casting_data.rank;
            let bonus_ad = attacker.borrow().get_bonus_ad();
            let base_ad = attacker.borrow().get_base_ad();

            let raw_damage = r_data.to_damage_amount(rank, base_ad, bonus_ad);

            target.receive_damage(&attacker.borrow(), raw_damage)
        };
    }

    pub fn auto_attack() -> impl Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) {
        return move |target: &mut Champion,
                     attacker: Rc<RefCell<Champion>>,
                     _casting_data: &CastingData| {
            let bonus_ad = attacker.borrow().get_bonus_ad();
            let base_ad = attacker.borrow().get_base_ad();

            let attack = BasicAttack::new(base_ad, bonus_ad);

            let raw_damage = attack.get_damage_to_target(
                &VitalityData::default(),
                &attacker.borrow().crit_info,
                None,
            );
            target.receive_damage(&attacker.borrow(), raw_damage);
            Vi::apply_w_effect(target, attacker);
        };
    }

    pub fn ult_combo(ranks: [u8; 4]) -> Vec<(ChampionAbilites, CastingData)> {
        //q , auto , e , (w), ult, auto, e
        let mut out = Vec::new();

        out.push((
            ChampionAbilites::Q,
            CastingData {
                rank: ranks[0],
                charge: Vi::Q_MAX_DAMAGE_CHARGE,
            },
        ));
        out.push((ChampionAbilites::AUTO, CastingData::new(0)));

        out.push((
            ChampionAbilites::E,
            CastingData {
                rank: ranks[2],
                charge: Vi::Q_MAX_DAMAGE_CHARGE,
            },
        ));

        out.push((
            ChampionAbilites::R,
            CastingData {
                rank: ranks[1],
                charge: Vi::Q_MAX_DAMAGE_CHARGE,
            },
        ));
        out.push((ChampionAbilites::AUTO, CastingData::new(0)));
        out.push((
            ChampionAbilites::E,
            CastingData {
                rank: ranks[2],
                charge: Vi::Q_MAX_DAMAGE_CHARGE,
            },
        ));

        return out;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        item_effects::{ChampionApplyable, ConcreteItemEffect},
        load_champion::ChampionStatModifier,
        load_wiki_item::{load_wiki_item_effects, load_wiki_item_stats},
        time_manager::TIME,
    };

    use super::*;
    use rstest::rstest;

    // values sampled from game on 13.7
    #[rstest]
    #[case(0, 0.0, 1, 0.0, 45)]
    #[case(0, Vi::Q_MAX_DAMAGE_CHARGE, 1, 0.0, 90)]
    #[case(1, Vi::Q_MAX_DAMAGE_CHARGE, 3, 30.0, 188)]
    fn test_abilty_q(
        #[case] rank: u8,
        #[case] charge: f64,
        #[case] level: u8,
        #[case] bonus_ad: f64,
        #[case] expected: u32,
    ) {
        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let mut vi = Champion::new(Vi::NAME.to_string(), level, [0, 0, 0, 0], vi_closures);
        vi.stats.bonus_attack_damage += bonus_ad;

        let target = &mut Champion::new_dummy();

        Champion::execute_ability(
            Rc::downgrade(&Rc::new(RefCell::new(vi))),
            &ChampionAbilites::Q,
            target,
            &CastingData { rank, charge },
        );
        assert_eq!(expected, target.get_missing_health().round() as u32);
    }

    #[rstest]
    fn test_full_combo() {
        let level = 6;

        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let mut vi = Champion::new(Vi::NAME.to_string(), level, [0, 0, 2, 0], vi_closures);

        vi.stats.bonus_attack_damage += 40.0;
        let target = &mut Champion::new_dummy();
        let ranks = vi.ranks;
        Champion::execute_combo(Rc::new(RefCell::new(vi)), Vi::ult_combo(ranks), target);
        assert_eq!(965, target.get_missing_health().round() as u32);
        // 905 dirk last whisper 30 armor
    }

    #[rstest]
    fn test_full_combo_2() {
        let level = 6;

        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let mut vi = Champion::new(Vi::NAME.to_string(), level, [0, 0, 2, 0], vi_closures);

        let item_names = ["Serrated Dirk", "Last Whisper"];
        for item_name in item_names {
            let item = load_wiki_item_stats(item_name.to_string());

            let concrete_item_effects: Vec<ConcreteItemEffect> =
                load_wiki_item_effects(item_name.to_string())
                    .iter()
                    .map(|v| v.into())
                    .collect();
            concrete_item_effects
                .iter()
                .for_each(|v| v.apply_to_champ(&mut vi));
            item.modify_champion_stats(&mut vi.stats);
        }

        let target = &mut Champion::new_dummy_with_resist(30.0, 0.0);
        let ranks = vi.ranks;
        Champion::execute_combo(Rc::new(RefCell::new(vi)), Vi::ult_combo(ranks), target);

        assert_eq!(905, target.get_missing_health().round() as u32);
    }

    #[rstest]
    fn test_w_via_autos() {
        let target = &mut Champion::new_dummy_with_resist(30.0, 0.0);

        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let vi = Rc::new(RefCell::new(Champion::new(
            Vi::NAME.to_string(),
            6,
            [0, 0, 2, 0],
            vi_closures,
        )));

        const HITS: usize = 9;
        let mut missing_healths: [f64; HITS] = [0.0; HITS];
        for i in 0..HITS {
            Champion::execute_ability(
                Rc::downgrade(&vi),
                &ChampionAbilites::AUTO,
                target,
                &CastingData {
                    ..Default::default()
                },
            );
            TIME.with(|time| *time.borrow_mut() += 1.0);

            missing_healths[i] = target.get_missing_health();
        }

        let mut damage: [f64; HITS] = [0.0; HITS];
        damage[0] = missing_healths[0];
        for i in 1..HITS {
            damage[i] = missing_healths[i] - missing_healths[i - 1];
        }
        assert_eq!(
            damage[1], damage[0],
            "first two autos should do equal damage {:#?}",
            damage
        );
        assert!(
            damage[2] > damage[1],
            "third hit does bonus damage {:#?}",
            damage
        );
        assert!(
            damage[4] > damage[1],
            "fourth hit does more damage than first/second {:#?}",
            damage
        );
        assert_eq!(
            damage[8], damage[5],
            "second/third w proc does equal damage {:#?}",
            damage
        );
        assert!(
            damage[2] < damage[5],
            "first w proc does less than second {:#?}",
            damage
        );
    }
}

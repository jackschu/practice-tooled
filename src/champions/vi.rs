use std::collections::HashMap;

use crate::{
    armor_reducer::ArmorReducer,
    attack::BasicAttack,
    target::{EffectData, EffectResult, VitalityData},
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
            Box<dyn Fn(&mut Champion, &Champion, &CastingData) -> ()>,
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

    pub fn ability_q(q_data: AbiltyDamageInfo) -> impl Fn(&mut Champion, &Champion, &CastingData) {
        return move |target: &mut Champion, attacker: &Champion, casting_data: &CastingData| {
            const MAX_SCALE: f64 = 1.0;
            let rank = casting_data.rank;
            let percent_damage = MAX_SCALE.min(casting_data.charge * 0.10 / 0.125) + 1.0;
            let base_ad = attacker.get_base_ad();
            let bonus_ad = attacker.get_bonus_ad();

            let mut raw_damage = q_data.to_damage_amount(rank, base_ad, bonus_ad);
            raw_damage *= percent_damage;
            target.receive_damage(attacker, raw_damage);
        };
    }

    // TODO, probably have ability ranks povided at construction time

    pub fn ability_w(w_data: AbiltyDamageInfo) -> impl Fn(&mut Champion, &Champion, &CastingData) {
        return move |target: &mut Champion, attacker: &Champion, casting_data: &CastingData| {
            let bonus_ad = attacker.get_bonus_ad();
            let rank = casting_data.rank;
            let percent_health_dmg = 0.01 * w_data.target_max_health_ratio[rank as usize]
                + 0.01 * w_data.bonus_ad_ratio * bonus_ad;
            let raw_damage = percent_health_dmg * target.get_max_health();
            target.receive_damage(attacker, raw_damage);
            target.add_effect(EffectData {
                ttl: 4.0,
                result: EffectResult::ArmorReducer(ArmorReducer {
                    percent_armor_reduction: 20.0,
                    ..Default::default()
                }),
            });
        };
    }

    pub fn ability_e(e_data: AbiltyDamageInfo) -> impl Fn(&mut Champion, &Champion, &CastingData) {
        return move |target: &mut Champion, attacker: &Champion, casting_data: &CastingData| {
            let rank = casting_data.rank;
            let bonus_ad = attacker.get_bonus_ad();
            let base_ad = attacker.get_base_ad();
            let e_dmg = e_data.to_damage_amount(rank, base_ad, bonus_ad);
            let attack = BasicAttack::new(e_dmg, 0.0);
            let raw_damage =
                attack.get_damage_to_target(&VitalityData::default(), &attacker.crit_info, None);
            target.receive_damage(attacker, raw_damage);
        };
    }

    pub fn ability_r(r_data: AbiltyDamageInfo) -> impl Fn(&mut Champion, &Champion, &CastingData) {
        return move |target: &mut Champion, attacker: &Champion, casting_data: &CastingData| {
            let rank = casting_data.rank;
            let bonus_ad = attacker.get_bonus_ad();
            let base_ad = attacker.get_base_ad();

            let raw_damage = r_data.to_damage_amount(rank, base_ad, bonus_ad);

            target.receive_damage(attacker, raw_damage)
        };
    }

    pub fn auto_attack() -> impl Fn(&mut Champion, &Champion, &CastingData) {
        return move |target: &mut Champion, attacker: &Champion, _casting_data: &CastingData| {
            let bonus_ad = attacker.get_bonus_ad();
            let base_ad = attacker.get_base_ad();

            let attack = BasicAttack::new(base_ad, bonus_ad);

            let raw_damage =
                attack.get_damage_to_target(&VitalityData::default(), &attacker.crit_info, None);
            target.receive_damage(attacker, raw_damage);
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
            ChampionAbilites::W,
            CastingData {
                rank: ranks[1],
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
    use super::*;
    use rstest::rstest;

    // values sampled from game on 13.7
    #[rstest]
    fn test_abilty_q() {
        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let mut vi = Champion::new(1, Vi::NAME.to_string(), vi_closures);

        let target = &mut Champion::new_dummy();

        vi.execute_ability(
            ChampionAbilites::Q,
            target,
            &CastingData {
                rank: 0,
                charge: 0.0,
            },
        );
        assert_eq!(45, target.get_missing_health().round() as u32);

        target.full_heal();
        vi.execute_ability(
            ChampionAbilites::Q,
            target,
            &CastingData {
                rank: 0,
                charge: Vi::Q_MAX_DAMAGE_CHARGE,
            },
        );
        assert_eq!(90, target.get_missing_health().round() as u32);

        vi.level = 3;
        vi.stats.bonus_attack_damage += 30.0;

        target.full_heal();

        vi.execute_ability(
            ChampionAbilites::Q,
            target,
            &CastingData {
                rank: 1,
                charge: Vi::Q_MAX_DAMAGE_CHARGE,
            },
        );
        assert_eq!(188, target.get_missing_health().round() as u32);
    }

    #[rstest]
    fn test_full_combo() {
        let level = 6;

        let mut vi_data = Vi::new();
        let vi_closures = vi_data.get_name_closures();
        let mut vi = Champion::new(level, Vi::NAME.to_string(), vi_closures);

        vi.stats.bonus_attack_damage += 40.0;
        let target = &mut Champion::new_dummy();
        vi.execute_combo(Vi::ult_combo([0, 0, 2, 0]), target);
        assert_eq!(965, target.get_missing_health().round() as u32);
    }
}

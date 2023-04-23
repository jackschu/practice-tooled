use crate::{
    attack::{BasicAttack, CritAdjuster, CritCalculation},
    load_champion::{load_champion_stats, ChampionStats},
    target::Target,
};

pub struct Vi {
    pub level: u8,
    pub q_data: AbiltyDamageInfo,
    pub w_data: AbiltyDamageInfo,
    pub e_data: AbiltyDamageInfo,
    pub r_data: AbiltyDamageInfo,

    pub stats: ChampionStats,
}

#[derive(Default)]
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
    const NAME: &str = "Vi";

    // as of 13.7
    #[allow(dead_code)]
    const Q_CD: [f64; 5] = [12.0, 10.5, 9.0, 7.5, 6.0];
    const Q_DAMAGE: [f64; 5] = [45.0, 70.0, 95.0, 120.0, 145.0];
    const Q_MAX_DAMAGE_CHARGE: f64 = 1.25;

    const W_HP_SCALING: [f64; 5] = [4.0, 5.5, 7.0, 8.5, 10.0];

    const E_DAMAGE: [f64; 5] = [0.0, 15.0, 30.0, 45.0, 60.0];
    const R_DAMAGE: [f64; 5] = [150.0, 325.0, 350.0, 0.0, 0.0];

    // TODO, probably have ability ranks povided at construction time
    pub fn new(level: u8) -> Vi {
        Vi {
            level,
            q_data: AbiltyDamageInfo {
                base_damages: Vi::Q_DAMAGE,
                bonus_ad_ratio: 80.0,
                ..Default::default()
            },
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
            stats: load_champion_stats(Vi::NAME),
        }
    }

    pub fn get_base_ad(&self) -> f64 {
        BasicAttack::from((&load_champion_stats(Vi::NAME), self.level)).base_attack_damage
    }
    pub fn get_bonus_ad(&self) -> f64 {
        self.stats.bonus_attack_damage
    }

    pub fn ability_q(&self, rank: u8, charge_seconds: f64) -> f64 {
        const MAX_SCALE: f64 = 1.0;
        let percent_damage = MAX_SCALE.min(charge_seconds * 0.10 / 0.125) + 1.0;
        let base_ad = self.get_base_ad();
        let bonus_ad = self.get_bonus_ad();

        let mut out = self.q_data.to_damage_amount(rank, base_ad, bonus_ad);
        out *= percent_damage;
        return out;
    }

    pub fn ability_w(&self, rank: u8, target_max_health: f64) -> f64 {
        let bonus_ad = self.get_bonus_ad();
        let percent_health_dmg = 0.01 * self.w_data.target_max_health_ratio[rank as usize]
            + 0.01 * self.w_data.bonus_ad_ratio * bonus_ad;
        return percent_health_dmg * target_max_health;
    }

    pub fn ability_e(&self, rank: u8, crit_info: &Option<(&CritAdjuster, CritCalculation)>) -> f64 {
        let bonus_ad = self.get_bonus_ad();
        let base_ad = self.get_base_ad();
        let e_dmg = self.e_data.to_damage_amount(rank, base_ad, bonus_ad);
        let attack = BasicAttack::new(e_dmg, 0.0);
        return attack.get_damage_to_target(&Target::default(), crit_info, None);
    }

    pub fn ability_r(&self, rank: u8) -> f64 {
        let base_ad = self.get_base_ad();
        let bonus_ad = self.get_bonus_ad();
        return self.r_data.to_damage_amount(rank, base_ad, bonus_ad);
    }

    pub fn get_ult_combo_damage(
        &self,
        ranks: [u8; 4],
        target_max_health: f64,
        crit_info: &Option<(&CritAdjuster, CritCalculation)>,
    ) -> f64 {
        let base_ad = self.get_base_ad();
        let bonus_ad = self.get_bonus_ad();
        let attack = BasicAttack::new(base_ad, bonus_ad);
        //q , auto , e , (w), ult, auto, e
        let mut out = 0.0;
        out += self.ability_q(ranks[0], Vi::Q_MAX_DAMAGE_CHARGE);
        out += attack.get_damage_to_target(&Target::default(), crit_info, None);
        out += self.ability_e(ranks[2], crit_info);
        out += self.ability_w(ranks[1], target_max_health);
        out += self.ability_r(ranks[3]);
        out += attack.get_damage_to_target(&Target::default(), crit_info, None);
        out += self.ability_e(ranks[2], crit_info);
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
        let mut vi = Vi::new(1);
        assert_eq!(45, vi.ability_q(0, 0.0).round() as u32);
        assert_eq!(90, vi.ability_q(0, Vi::Q_MAX_DAMAGE_CHARGE).round() as u32);
        vi.level = 3;
        vi.stats.bonus_attack_damage += 30.0;
        assert_eq!(188, vi.ability_q(1, Vi::Q_MAX_DAMAGE_CHARGE).round() as u32);
    }

    #[rstest]
    fn test_full_combo() {
        let mut vi = Vi::new(6);
        vi.stats.bonus_attack_damage += 40.0;
        assert_eq!(
            965,
            vi.get_ult_combo_damage([0, 0, 2, 0], 1000.0, &None).round() as u32
        );
    }
}

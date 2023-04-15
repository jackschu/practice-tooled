use crate::{
    attack::{BasicAttack, CritAdjuster, CritCalculation, Target, TargetData},
    load_champion::load_champion_stats,
};

pub struct Vi {
    pub level: u8,
    pub q_data: AbiltyDamageInfo,
    pub e_data: AbiltyDamageInfo,
}

pub struct AbiltyDamageInfo {
    pub base_damages: [f64; 5],
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
    const Q_CD: [f64; 5] = [12.0, 10.5, 9.0, 7.5, 6.0];
    const Q_DAMAGE: [f64; 5] = [45.0, 70.0, 95.0, 120.0, 145.0];
    const Q_MAX_DAMAGE_CHARGE: f64 = 1.25;

    const E_DAMAGE: [f64; 5] = [0.0, 15.0, 30.0, 45.0, 60.0];

    // TODO, probably have ability ranks povided at construction time
    pub fn new(level: u8) -> Vi {
        Vi {
            level,
            q_data: AbiltyDamageInfo {
                base_damages: Vi::Q_DAMAGE,
                ad_ratio: 0.0,
                bonus_ad_ratio: 80.0,
            },
            e_data: AbiltyDamageInfo {
                base_damages: Vi::E_DAMAGE,
                ad_ratio: 120.0,
                bonus_ad_ratio: 0.0,
            },
        }
    }

    pub fn get_base_ad(&self) -> f64 {
        return load_champion_stats(Vi::NAME)
            .as_basic_attack(self.level)
            .base_attack_damage;
    }

    pub fn ability_q(&self, rank: u8, bonus_ad: f64, charge_seconds: f64) -> f64 {
        const MAX_SCALE: f64 = 1.0;
        let percent_damage = MAX_SCALE.min(charge_seconds * 0.10 / 0.125) + 1.0;
        let base_ad = self.get_base_ad();

        let mut out = self.q_data.to_damage_amount(rank, base_ad, bonus_ad);
        out *= percent_damage;
        return out;
    }

    pub fn ability_e(
        &self,
        rank: u8,
        bonus_ad: f64,
        crit_info: Option<(&CritAdjuster, CritCalculation)>,
    ) -> f64 {
        let base_ad = self.get_base_ad();
        let mut out = self.e_data.to_damage_amount(rank, base_ad, bonus_ad);
        let attack = BasicAttack::new(base_ad, bonus_ad);
        out += attack.get_damage_to_target(&Target::new(TargetData::default()), crit_info, None);
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
        assert_eq!(45, vi.ability_q(0, 0.0, 0.0).round() as u32);
        assert_eq!(
            90,
            vi.ability_q(0, 0.0, Vi::Q_MAX_DAMAGE_CHARGE).round() as u32
        );
        vi.level = 3;
        assert_eq!(
            188,
            vi.ability_q(1, 30.0, Vi::Q_MAX_DAMAGE_CHARGE).round() as u32
        );
    }
}

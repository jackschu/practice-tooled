use crate::{
    attack::{BasicAttack, CritAdjuster, CritCalculation},
    core::stat_at_level,
    load_champion::{load_champion_stats, ChampionStats},
    target::VitalityData,
};

use super::champion::Champion;

impl Champion for Vi {
    fn get_stats_mut(&mut self) -> &mut ChampionStats {
        let out = &mut self.stats;
        return out;
    }
    fn get_level(&self) -> u8 {
        self.level
    }
    fn get_stats(&self) -> &ChampionStats {
        &self.stats
    }

    fn get_current_health(&self) -> f64 {
        self.current_health
    }
    fn get_current_health_mut(&mut self) -> &mut f64 {
        return &mut self.current_health;
    }
    fn get_initial_armor(&self) -> f64 {
        self.initial_armor
    }
}

pub struct Vi {
    pub level: u8,
    pub q_data: AbiltyDamageInfo,
    pub w_data: AbiltyDamageInfo,
    pub e_data: AbiltyDamageInfo,
    pub r_data: AbiltyDamageInfo,
    pub stats: ChampionStats,
    initial_armor: f64, // base armor before level ups
    current_health: f64,
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
        let stats = load_champion_stats(Vi::NAME.to_string());
        let health = stat_at_level(stats.health, stats.health_per_level, level);
        let initial_armor = stats.armor;
        return Vi {
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
            stats,
            initial_armor,
            current_health: health,
        };
    }

    pub fn get_base_ad(&self) -> f64 {
        BasicAttack::from((&self.stats, self.level)).base_attack_damage
    }

    pub fn get_bonus_ad(&self) -> f64 {
        self.stats.bonus_attack_damage
    }

    pub fn ability_q(&self, rank: u8, target: &mut dyn Champion, charge_seconds: f64) {
        const MAX_SCALE: f64 = 1.0;
        let percent_damage = MAX_SCALE.min(charge_seconds * 0.10 / 0.125) + 1.0;
        let base_ad = self.get_base_ad();
        let bonus_ad = self.get_bonus_ad();

        let mut raw_damage = self.q_data.to_damage_amount(rank, base_ad, bonus_ad);
        raw_damage *= percent_damage;
        target.receive_damage(self, raw_damage);
    }

    pub fn ability_w(&self, rank: u8, target: &mut dyn Champion) {
        let bonus_ad = self.get_bonus_ad();
        let percent_health_dmg = 0.01 * self.w_data.target_max_health_ratio[rank as usize]
            + 0.01 * self.w_data.bonus_ad_ratio * bonus_ad;
        let raw_damage = percent_health_dmg * target.get_max_health();
        target.receive_damage(self, raw_damage);
    }

    pub fn ability_e(
        &self,
        rank: u8,
        target: &mut dyn Champion,
        crit_info: &Option<(&CritAdjuster, CritCalculation)>,
    ) {
        let bonus_ad = self.get_bonus_ad();
        let base_ad = self.get_base_ad();
        let e_dmg = self.e_data.to_damage_amount(rank, base_ad, bonus_ad);
        let attack = BasicAttack::new(e_dmg, 0.0);
        let raw_damage = attack.get_damage_to_target(&VitalityData::default(), crit_info, None);
        target.receive_damage(self, raw_damage)
    }

    pub fn ability_r(&self, rank: u8, target: &mut dyn Champion) {
        let base_ad = self.get_base_ad();
        let bonus_ad = self.get_bonus_ad();
        let raw_damage = self.r_data.to_damage_amount(rank, base_ad, bonus_ad);

        target.receive_damage(self, raw_damage)
    }

    pub fn auto_attack(
        &self,
        target: &mut dyn Champion,
        crit_info: &Option<(&CritAdjuster, CritCalculation)>,
    ) {
        let base_ad = self.get_base_ad();
        let bonus_ad = self.get_bonus_ad();
        let attack = BasicAttack::new(base_ad, bonus_ad);

        let raw_damage = attack.get_damage_to_target(&VitalityData::default(), crit_info, None);
        target.receive_damage(self, raw_damage)
    }
    pub fn ult_combo(
        &self,
        ranks: [u8; 4],
        target: &mut dyn Champion,
        crit_info: &Option<(&CritAdjuster, CritCalculation)>,
    ) {
        //q , auto , e , (w), ult, auto, e
        self.ability_q(ranks[0], target, Vi::Q_MAX_DAMAGE_CHARGE);
        self.auto_attack(target, crit_info);
        self.ability_e(ranks[2], target, crit_info);
        self.ability_w(ranks[1], target);
        self.ability_r(ranks[3], target);
        self.auto_attack(target, crit_info);
        self.ability_e(ranks[2], target, crit_info);
    }
}

#[cfg(test)]
mod tests {
    use crate::champions::target_dummy::TargetDummy;

    use super::*;
    use rstest::rstest;

    // values sampled from game on 13.7
    #[rstest]
    fn test_abilty_q() {
        let mut vi = Vi::new(1);

        let target = &mut TargetDummy::new();
        vi.ability_q(0, target, 0.0);
        assert_eq!(45, target.get_missing_health().round() as u32);

        target.full_heal();
        vi.ability_q(0, target, Vi::Q_MAX_DAMAGE_CHARGE);
        assert_eq!(90, target.get_missing_health().round() as u32);

        vi.level = 3;
        vi.stats.bonus_attack_damage += 30.0;

        target.full_heal();
        vi.ability_q(1, target, Vi::Q_MAX_DAMAGE_CHARGE);
        assert_eq!(188, target.get_missing_health().round() as u32);
    }

    #[rstest]
    fn test_full_combo() {
        let mut vi = Vi::new(6);
        vi.stats.bonus_attack_damage += 40.0;
        let target = &mut TargetDummy::new();
        vi.ult_combo([0, 0, 2, 0], target, &None);
        assert_eq!(965, target.get_missing_health().round() as u32);
    }
}

use crate::load_champion::load_champion_stats;

pub struct Vi {
    pub level: u8,
    pub q_data: AbiltyDamageInfo,
}

pub struct AbiltyDamageInfo {
    pub base_damages: [f64; 5],
    pub ad_ratio: f64,
    pub bonus_ad_ratio: f64,
}

impl AbiltyDamageInfo {
    pub fn to_damage_amount(&self, rank: u8, base: f64, bonus: f64) -> f64 {
        return self.base_damages[rank as usize]
            + self.ad_ratio * (base + bonus)
            + self.bonus_ad_ratio * bonus;
    }
}

impl Vi {
    // as of 13.7
    const Q_CD: [f64; 5] = [12.0, 10.5, 9.0, 7.5, 6.0];
    const Q_DAMAGE: [f64; 5] = [45.0, 70.0, 95.0, 120.0, 145.0];

    pub fn new(level: u8) -> Vi {
        Vi {
            level,
            q_data: AbiltyDamageInfo {
                base_damages: Vi::Q_DAMAGE,
                ad_ratio: 80.0,
                bonus_ad_ratio: 80.0,
            },
        }
    }

    pub fn ability_q(&self, charge_seconds: f64) -> f64 {
        const MAX_SCALE: f64 = 0.5;
        let percent_damage = MAX_SCALE.min(charge_seconds * 0.05 / 0.125) + 0.5;
        let champion = load_champion_stats("Vi").as_basic_attack(self.level);

        let mut out = self
            .q_data
            .to_damage_amount(1, champion.base_attack_damage, 0.0);
        out *= percent_damage;
        return out;
    }
}

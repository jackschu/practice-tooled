use crate::load_champion::load_champion_stats;

use super::super::attack;

pub struct Vi {
    pub level: u8,
    pub q_data: SingleDamage,
}

pub struct SingleDamage {
    pub damages: [f64; 5],
    pub ad_ratio: f64,
}

impl SingleDamage {
    pub fn to_basic_attack(
        &self,
        rank: u8,
        unqualified_attack: &attack::BasicAttack,
    ) -> attack::BasicAttack {
        let mut out = unqualified_attack.clone();
        out.attack_damage = self.damages[rank as usize] + self.ad_ratio * out.attack_damage;
        return out;
    }
}
impl Vi {
    // as of 13.7
    const Q_CD: [f64; 5] = [12.0, 10.5, 9.0, 7.5, 6.0];
    const Q_DAMAGE: [f64; 5] = [45.0, 70.0, 95.0, 120.0, 145.0];

    pub fn new(level: u8) -> Vi {
        Vi {
            level,
            q_data: SingleDamage {
                damages: Vi::Q_DAMAGE,
                ad_ratio: 80.0,
            },
        }
    }

    pub fn ability_q(&self, charge_seconds: f64) -> attack::BasicAttack {
        const MAX_SCALE: f64 = 0.5;
        let percent_damage = MAX_SCALE.min(charge_seconds * 0.05 / 0.125) + 0.5;
        let champion = load_champion_stats("Vi").as_basic_attack(self.level);

        let mut out = self.q_data.to_basic_attack(1, &champion);
        out.attack_damage *= percent_damage;
        return out;
    }
}

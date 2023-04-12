use std::cmp;

pub struct Vi {
    pub level: u8,
    pub q_data: SingleDamage,
}

pub struct SingleDamage {
    pub damages: [f64; 5],
    pub ad_ratio: f64,
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
}

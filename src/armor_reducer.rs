use crate::{core::lethality_to_pen, load_champion::ChampionStats, target::Target};

#[derive(Default, Clone)]
pub struct ArmorReducer {
    pub flat_armor_reduction: f64,
    pub percent_armor_reduction: f64,

    pub flat_armor_pen: f64, // effective lethality
    pub percent_armor_pen: f64,
    pub percent_bonus_armor_pen: f64,
}

impl ArmorReducer {
    pub fn apply_armor_reduction(&self, target: &mut Target) {
        let total_armor = target.base_armor + target.bonus_armor;
        let base_ratio = if total_armor != 0.0 {
            target.base_armor / total_armor
        } else {
            0.5
        };
        let bonus_ratio = 1.0 - base_ratio;
        target.base_armor -= self.flat_armor_reduction * base_ratio;
        target.bonus_armor -= self.flat_armor_reduction * bonus_ratio;

        if target.base_armor > 0.0 {
            target.base_armor *= 1.0 - self.percent_armor_reduction / 100.0;
        }
        if target.bonus_armor > 0.0 {
            target.bonus_armor *= 1.0 - self.percent_armor_reduction / 100.0;
        }
    }

    pub fn get_effective_armor(&self, original_target: &Target) -> f64 {
        let mut target = original_target.clone();
        self.apply_armor_reduction(&mut target);
        let mut effective_armor =
            target.bonus_armor * (1.0 - self.percent_bonus_armor_pen / 100.0) + target.base_armor;
        effective_armor *= 1.0 - self.percent_armor_pen / 100.0;

        // lethality can't reduce below 0
        if effective_armor < 0.0 {
            return effective_armor;
        }
        let candidate = effective_armor - self.flat_armor_pen;
        if candidate < 0.0 {
            return 0.0;
        } else {
            return candidate;
        }
    }

    pub fn set_from_lethality(&mut self, lethality: f64, level: u8) {
        self.flat_armor_pen = lethality_to_pen(lethality, level);
    }
}

impl From<(&ChampionStats, u8)> for ArmorReducer {
    fn from(tuple: (&ChampionStats, u8)) -> ArmorReducer {
        let (stats, level) = tuple;
        return ArmorReducer {
            flat_armor_pen: lethality_to_pen(stats.lethality, level),
            percent_armor_pen: stats.percent_armor_pen,
            ..Default::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rstest::rstest;

    #[rstest]
    // flat reduction wiki example
    #[case(20.0, 40.0, ArmorReducer{ flat_armor_reduction: 30.0, ..Default::default() } , 30.0)]
    // percent reduction wiki example
    #[case(20.0, 40.0, ArmorReducer{ percent_armor_reduction: 30.0, ..Default::default() } , 42.0)]
    // percent pen wiki example
    #[case(20.0, 40.0, ArmorReducer{ percent_armor_pen: 30.0, ..Default::default() } , 42.0)]
    // percent bonus pen wiki example
    #[case(20.0, 40.0, ArmorReducer{
		percent_armor_pen: 10.0,
		percent_bonus_armor_pen: 30.0,
		..Default::default() } , 43.2
	)]
    // high level wiki example
    #[case(100.0, 200.0, ArmorReducer{
		percent_bonus_armor_pen: 45.0,
		flat_armor_pen: 10.0,
		flat_armor_reduction: 30.0,
		percent_armor_reduction: 30.0,
		..Default::default()
	} , 122.3)]
    // high level wiki example
    #[case(18.0, 0.0, ArmorReducer{
		percent_bonus_armor_pen: 45.0,
		flat_armor_pen: 10.0,
		flat_armor_reduction: 30.0,
		percent_armor_reduction: 30.0,
		..Default::default()
	} , -12.0)]
    fn effective_armor(
        #[case] base_armor: f64,
        #[case] bonus_armor: f64,
        #[case] reducer: ArmorReducer,
        #[case] expected_armor: f64,
    ) {
        let target = Target {
            base_armor,
            bonus_armor,
            ..Default::default()
        };

        assert_relative_eq!(expected_armor, reducer.get_effective_armor(&target));
    }
}

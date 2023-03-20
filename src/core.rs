pub fn resist_damage(raw_damage: f64, resist_amount: f64) -> f64 {
    const SCALING_RATIO: f64 = 100.0;
    if resist_amount > 0.0 {
        return raw_damage * SCALING_RATIO / (SCALING_RATIO + resist_amount);
    }

    return raw_damage * (2.0 - SCALING_RATIO / (SCALING_RATIO - resist_amount));
}

pub fn lethality_to_pen(lethality: f64, level: u32) -> f64 {
    return lethality * (0.6 + 0.4 * (level as f64) / 18.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use approx::assert_relative_eq;
    #[rstest]
    #[case(1532.0, 0.0, 1532.0)] // zero resist is true
    #[case(1000.0, 25.0, 800.0)] // positive resist
    #[case(1000.0, -100.0, 1500.0)] // negative resist
    fn test_resist_damage(#[case] damage: f64, #[case] resist: f64, #[case] expected_damage: f64) {
        assert_relative_eq!(expected_damage, resist_damage(damage, resist))
    }
    #[rstest]
    #[case(20.0, 18, 20.0)]
    #[case(100.0, 1, 560.0/9.0)]
    fn test_lethality_to_pen(#[case] lethality: f64, #[case] level: u32, #[case] expected: f64) {
        assert_relative_eq!(expected, lethality_to_pen(lethality, level))
    }
}

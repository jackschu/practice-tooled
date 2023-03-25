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

pub fn stat_at_level(base: f64, growth: f64, level: f64) -> f64 {
    return base + growth * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0));
}

pub fn haste_to_cdr(mut haste: f64) -> f64 {
    if haste > 500.0 {
        haste = 500.0;
    }
    return haste / (haste + 100.0) * 100.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rstest::rstest;
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
    #[rstest]
    #[case(670.0, 120.0, 2.0, 756.4)]
    #[case(0.0, 1.0, 18.0, 17.0)]
    fn test_stat_at_level(
        #[case] base: f64,
        #[case] growth: f64,
        #[case] level: f64,
        #[case] expected: f64,
    ) {
        assert_relative_eq!(expected, stat_at_level(base, growth, level))
    }
    #[rstest]
    #[case(100.0, 50.0)]
    #[case(544.0, 250.0/3.0)] // test cdr past 500 cap
    fn test_haste_to_cdr(#[case] haste: f64, #[case] expected: f64) {
        assert_relative_eq!(expected, haste_to_cdr(haste))
    }
}

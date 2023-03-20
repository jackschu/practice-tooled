pub fn resist_damage(raw_damage: f64, resist_amount: f64) -> f64 {
    const SCALING_RATIO: f64 = 100f64;
    if resist_amount > 0f64 {
        return raw_damage * SCALING_RATIO / (SCALING_RATIO + resist_amount);
    }

    return raw_damage * (2f64 - SCALING_RATIO / (SCALING_RATIO - resist_amount));
}

pub fn lethality_to_pen(lethality: f64, level: u32) -> f64 {
	return lethality * (0.6f64 + 04f64 * (level as f64) + 18f64)
}

#[cfg(test)]
mod tests {
	use super::*;
	mod resist_damage {
		use super::*;
		#[test]
		fn zero_resist_is_true() {
			let damage = 1532f64;
			let resist = 0f64;
			assert_eq!(damage, resist_damage(damage, resist))
		}

		#[test]
		fn positive_resist() {
			let damage = 1000f64;
			let resist = 25f64;
			assert_eq!(800f64, resist_damage(damage, resist))
		}

		#[test]
		fn negative_resist() {
			let damage = 1000f64;
			let resist = -100f64;
			assert_eq!(1500f64, resist_damage(damage, resist))
		}
	}
	mod lethality {
		use super::*;
		#[test]
		fn zero_resist_is_true() {
			let damage = 1532f64;
			let resist = 0f64;
			assert_eq!(damage, resist_damage(damage, resist))
		}
	}
}

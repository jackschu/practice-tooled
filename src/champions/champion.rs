use crate::{
    armor_reducer::ArmorReducer,
    core::{resist_damage, stat_at_level},
    load_champion::ChampionStats,
    target::{Target, VitalityData},
};

pub trait Champion {
    fn get_stats_mut(&mut self) -> &mut ChampionStats;
    fn get_stats(&self) -> &ChampionStats;
    fn get_health_mut(&mut self) -> &mut f64;
    fn get_current_health(&self) -> f64;
    fn get_level(&self) -> u8;

    fn get_initial_armor(&self) -> f64;
    fn get_base_armor(&self) -> f64 {
        let stats = self.get_stats();
        return stat_at_level(
            self.get_initial_armor(),
            stats.armor_per_level,
            self.get_level(),
        );
    }
    fn get_bonus_armor(&self) -> f64 {
        let stats = self.get_stats();
        return stats.armor - self.get_initial_armor();
    }

    fn get_max_health(&self) -> f64 {
        let stats = self.get_stats();
        return stat_at_level(stats.health, stats.health_per_level, self.get_level());
    }
    fn get_magic_resist(&self) -> f64 {
        let stats = self.get_stats();
        return stat_at_level(
            stats.magic_resist,
            stats.magic_resist_per_level,
            self.get_level(),
        );
    }

    fn receive_damage(&mut self, attacker: &dyn Champion, damage: f64) {
        let armor_reducer: ArmorReducer = (attacker.get_stats(), attacker.get_level()).into();
        let target_data = self.get_vitality_data();
        let effective_armor = armor_reducer.get_effective_armor(&target_data);
        let final_damage = resist_damage(damage, effective_armor);
        let health = self.get_health_mut();
        *health = *health - final_damage;
    }
}

impl<T: ?Sized + Champion> Target for T {
    fn get_vitality_data(&self) -> VitalityData {
        return VitalityData {
            base_armor: self.get_base_armor(),
            bonus_armor: self.get_bonus_armor(),
            magic_resist: self.get_magic_resist(),
            max_health: self.get_max_health(),
            current_health: self.get_current_health(),
        };
    }
}

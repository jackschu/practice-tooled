use crate::{
    core::stat_at_level,
    load_champion::{load_champion_stats, ChampionStats},
};

use super::champion::Champion;

impl Champion for Leblanc {
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

pub struct Leblanc {
    pub level: u8,
    pub stats: ChampionStats,
    initial_armor: f64, // base armor before level ups
    current_health: f64,
}

impl Leblanc {
    const NAME: &str = "Leblanc";

    pub fn new(level: u8) -> Leblanc {
        let stats = load_champion_stats(Leblanc::NAME.to_string());
        let health = stat_at_level(stats.health, stats.health_per_level, level);
        let initial_armor = stats.armor;
        return Leblanc {
            stats,
            initial_armor,
            current_health: health,
            level,
        };
    }
}

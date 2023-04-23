use crate::load_champion::ChampionStats;

use super::champion::Champion;

impl Champion for TargetDummy {
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

pub struct TargetDummy {
    pub level: u8,
    pub stats: ChampionStats,
    initial_armor: f64, // base armor before level ups
    current_health: f64,
}

impl TargetDummy {
    pub fn new() -> TargetDummy {
        TargetDummy::new_with_resist(0.0, 0.0)
    }
    pub fn new_with_resist(armor: f64, magic_resist: f64) -> TargetDummy {
        let health = 1000.0;
        let stats = ChampionStats {
            health,
            armor,
            magic_resist,
            ..Default::default()
        };

        return TargetDummy {
            stats,
            initial_armor: armor,
            current_health: health,
            level: 1,
        };
    }
    pub fn full_heal(&mut self) {
        self.current_health = self.get_max_health()
    }
}

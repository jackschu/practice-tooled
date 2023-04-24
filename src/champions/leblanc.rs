use crate::{
    core::stat_at_level,
    load_champion::{load_champion_stats, ChampionStats},
};

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

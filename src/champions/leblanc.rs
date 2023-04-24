use crate::load_champion::{load_champion_stats, ChampionStats};

pub struct Leblanc {
    pub level: u8,
    pub stats: ChampionStats,
}

impl Leblanc {
    pub const NAME: &str = "Leblanc";

    pub fn new(level: u8) -> Leblanc {
        let stats = load_champion_stats(Leblanc::NAME.to_string());

        return Leblanc { stats, level };
    }
}

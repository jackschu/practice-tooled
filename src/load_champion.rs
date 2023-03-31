use serde::Deserialize;
use serde::Serialize;
use serde_json;
use serde_json::Value;

use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct ChampionStats {
    #[serde(rename = "hp")]
    pub health: f64,
    #[serde(rename = "hpperlevel")]
    pub health_per_level: f64,
    #[serde(rename = "mp")]
    pub mana: f64,
    #[serde(rename = "mpperlevel")]
    pub mana_per_level: f64,
    #[serde(rename = "movespeed")]
    pub move_speed: f64,
    #[serde(rename = "armor")]
    pub armor: f64,
    #[serde(rename = "armorperlevel")]
    pub armor_per_level: f64,
    #[serde(rename = "spellblock")]
    pub magic_resist: f64,
    #[serde(rename = "spellblockperlevel")]
    pub magic_resist_per_level: f64,
    #[serde(rename = "attackrange")]
    pub attack_range: f64,
    #[serde(rename = "hpregen")]
    pub health_regen: f64,
    #[serde(rename = "hpregenperlevel")]
    pub health_regen_per_level: f64,
    #[serde(rename = "mpregen")]
    pub mana_regen: f64,
    #[serde(rename = "mpregenperlevel")]
    pub mana_regen_per_level: f64,
    #[serde(rename = "crit")]
    pub critical_strike_chance: f64,
    #[serde(rename = "critperlevel")]
    pub crit_per_level: f64,
    #[serde(rename = "attackdamage")]
    pub attack_damage: f64,
    #[serde(rename = "attackdamageperlevel")]
    pub attack_damage_per_level: f64,
    #[serde(rename = "attackspeedperlevel")]
    pub attack_speed_per_level: f64,
    #[serde(rename = "attackspeed")]
    pub attack_speed: f64,
}

pub fn load_champion_stats(champion_name: &str) -> ChampionStats {
    let mut file = File::open("data/champion.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    let data = full_value
        .get("data")
        .and_then(|value| value.get(champion_name))
        .and_then(|value| value.get("stats"))
        .expect("could not index to champ");
    // why is this clone needed?
    let champion_stats: ChampionStats =
        serde_json::from_value(data.clone()).expect("could not unmarshal to person");
    return champion_stats;
}

#[cfg(test)]
mod tests {
    use super::load_champion_stats;

    use rstest::rstest;

    #[rstest]
    fn test_can_load_sivir() {
        let stats = load_champion_stats("Sivir");
        assert_eq!(stats.critical_strike_chance, 0.0);
    }
}

use memoize::memoize;

use serde::Deserialize;
use serde::Serialize;
use serde_json;
use serde_json::Value;

use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::option::Option;

#[derive(Deserialize, Serialize, Clone, Default)]
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
    #[serde(default)]
    pub bonus_attack_damage: f64,
    #[serde(rename = "attackdamage")]
    pub base_attack_damage: f64,
    #[serde(rename = "attackdamageperlevel")]
    pub attack_damage_per_level: f64,
    #[serde(rename = "attackspeedperlevel")]
    pub attack_speed_per_level: f64,
    #[serde(rename = "attackspeed")]
    pub attack_speed: f64,
    #[serde(skip)]
    pub bonus_attack_speed: f64,
    #[serde(skip)]
    pub life_steal: f64,
    #[serde(skip)]
    pub percent_movement_speed: f64,
    #[serde(skip)]
    pub ability_haste: f64,
    #[serde(skip)]
    pub omnivamp: f64,
    #[serde(skip)]
    pub ability_power: f64,
    #[serde(skip)]
    pub percent_armor_pen: f64,
    #[serde(skip)]
    pub lethality: f64,
}

pub trait ChampionStatModifier: Debug {
    fn modify_champion_stats(&self, stats: &mut ChampionStats);
}

#[memoize]
pub fn open_champion_json() -> Option<Value> {
    let mut file = File::open("data/champion.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    return full_value.get("data").map(|v| v.to_owned());
}

pub fn load_champion_names() -> Vec<String> {
    let data = open_champion_json().unwrap();
    let mut names = Vec::new();
    if let Value::Object(map) = data {
        for (name, _) in map {
            names.push(name);
        }
    }
    return names;
}

pub fn load_champion_stats(champion_name: &str) -> ChampionStats {
    let data = open_champion_json().unwrap();
    let champion_stats_json = data
        .get(champion_name)
        .and_then(|value| value.get("stats"))
        .unwrap();

    // why is this clone needed?
    let champion_stats: ChampionStats =
        serde_json::from_value(champion_stats_json.clone()).unwrap();
    return champion_stats;
}

#[cfg(test)]
mod tests {
    use crate::attack::BasicAttack;

    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_can_load_sivir() {
        let stats = load_champion_stats("Sivir");
        assert_eq!(stats.critical_strike_chance, 0.0);
    }

    #[rstest]
    fn test_load_champion_basic_attack() {
        let stats = load_champion_stats("Vi");
        let attack: BasicAttack = (&stats, 5).into();
        assert_eq!(72.0, attack.base_attack_damage.round()); // values from game, patch 13.6
    }
}

use crate::attack::AttackSpeed;
use crate::load_item::ItemStatDeltas;

use super::attack::BasicAttack;
use super::attack::Target;
use super::core;

use memoize::memoize;

use serde::Deserialize;
use serde::Serialize;
use serde_json;
use serde_json::Value;

use std::fs::File;
use std::io::prelude::*;
use std::option::Option;

#[derive(Deserialize, Serialize, Clone)]
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
}

impl ChampionStats {
    pub fn add_item_deltas(&mut self, item: &ItemStatDeltas) {
        self.armor += item.armor.unwrap_or(0.0);
        self.magic_resist += item.magic_resist.unwrap_or(0.0);
        self.health_regen += item.health_regen.unwrap_or(0.0);
        self.health += item.health.unwrap_or(0.0);
        self.mana += item.mana.unwrap_or(0.0);
        self.bonus_attack_damage += item.attack_damage.unwrap_or(0.0);
        self.bonus_attack_speed += item.bonus_attack_speed.unwrap_or(0.0);
        self.life_steal += item.life_steal.unwrap_or(0.0);
        self.percent_movement_speed += item.percent_movement_speed.unwrap_or(0.0);
        self.move_speed += item.flat_movement_speed.unwrap_or(0.0);
    }
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

impl ChampionStats {
    pub fn as_basic_attack(&self, level: u8) -> BasicAttack {
        let attack_damage =
            core::stat_at_level(self.base_attack_damage, self.attack_damage_per_level, level)
                + self.bonus_attack_damage;
        let attack = BasicAttack {
            base_attack_damage: attack_damage,
            ..Default::default()
        };

        return attack;
    }

    pub fn as_attack_speed(&self, level: u8) -> AttackSpeed {
        let bonus_speed =
            core::stat_at_level(0.0, self.attack_speed_per_level, level) + self.bonus_attack_speed;
        return AttackSpeed {
            base: self.attack_speed,
            bonus: bonus_speed,
        };
    }

    pub fn as_target(&self, level: u8) -> Target {
        let base_armor = core::stat_at_level(self.armor, self.armor_per_level, level);
        let magic_resist =
            core::stat_at_level(self.magic_resist, self.magic_resist_per_level, level);
        let max_health = core::stat_at_level(self.health, self.health_per_level, level);
        let target = Target {
            base_armor,
            bonus_armor: 0.0,
            max_health,
            current_health: max_health,
            magic_resist,
        };

        return target;
    }
}

#[cfg(test)]
mod tests {
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
        let attack = stats.as_basic_attack(5);
        assert_eq!(72.0, attack.base_attack_damage.round()); // values from game, patch 13.6
    }
}

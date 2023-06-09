use std::{collections::HashMap, fs::File, io::Read};

use memoize::memoize;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use super::load_champion::ChampionStats;

use super::load_champion::ChampionStatModifier;

const SUMMONERS_RIFT_MAP_ID: &str = "11";

#[derive(Deserialize, Serialize, Debug)]
pub struct DDItemStatDeltas {
    #[serde(rename = "FlatArmorMod")]
    pub armor: Option<f64>,
    #[serde(rename = "FlatCritChanceMod")]
    pub crit_chance: Option<f64>,
    #[serde(rename = "FlatHPPoolMod")]
    pub health: Option<f64>,
    #[serde(rename = "FlatHPRegenMod")]
    pub health_regen: Option<f64>,
    #[serde(rename = "FlatMagicDamageMod")]
    pub ability_power: Option<f64>,
    #[serde(rename = "FlatMovementSpeedMod")]
    pub flat_movement_speed: Option<f64>,
    #[serde(rename = "FlatMPPoolMod")]
    pub mana: Option<f64>,
    #[serde(rename = "FlatPhysicalDamageMod")]
    pub attack_damage: Option<f64>,
    #[serde(rename = "FlatSpellBlockMod")]
    pub magic_resist: Option<f64>,
    #[serde(rename = "PercentAttackSpeedMod")]
    pub bonus_attack_speed: Option<f64>,
    #[serde(rename = "PercentLifeStealMod")]
    pub life_steal: Option<f64>,
    #[serde(rename = "PercentMovementSpeedMod")]
    pub percent_movement_speed: Option<f64>,
}

#[memoize]
pub fn open_dd_item_json() -> Value {
    let mut file = File::open("data/item.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    return full_value.get("data").map(|v| v.to_owned()).unwrap();
}

fn name_to_id_map() -> HashMap<String, String> {
    let mut output_map = HashMap::new();
    let all_items = load_items();
    all_items.iter().for_each(|(key, value)| {
        let name = value.get("name").and_then(|v| v.as_str()).unwrap();
        output_map.entry(name.to_string()).or_insert(key.to_owned());
        return;
    });
    return output_map;
}

pub fn load_dd_item(name: &str) -> DDItemStatDeltas {
    let map = name_to_id_map();
    let id = map.get(name).unwrap();
    let item_value = load_items()
        .get(id)
        .and_then(|v| v.get("stats"))
        .unwrap()
        .clone();
    let item_obj: DDItemStatDeltas = serde_json::from_value(item_value).unwrap();
    return item_obj;
}

impl ChampionStatModifier for DDItemStatDeltas {
    fn modify_champion_stats(&self, stats: &mut ChampionStats) {
        stats.armor += self.armor.unwrap_or(0.0);
        stats.magic_resist += self.magic_resist.unwrap_or(0.0);
        stats.health_regen += self.health_regen.unwrap_or(0.0);
        stats.health += self.health.unwrap_or(0.0);
        stats.mana += self.mana.unwrap_or(0.0);
        stats.bonus_attack_damage += self.attack_damage.unwrap_or(0.0);
        stats.bonus_attack_speed += self.bonus_attack_speed.unwrap_or(0.0);
        stats.life_steal += self.life_steal.unwrap_or(0.0);
        stats.percent_movement_speed += self.percent_movement_speed.unwrap_or(0.0);
        stats.move_speed += self.flat_movement_speed.unwrap_or(0.0);
    }
}

fn load_items() -> serde_json::Map<std::string::String, Value> {
    let json_value = open_dd_item_json();
    let mut filtered_items = json_value.as_object().unwrap().clone();
    filtered_items.retain(|_key, value| {
        let purchasable = value
            .get("gold")
            .and_then(|v| v.get("purchasable"))
            .and_then(|v| v.as_bool())
            .unwrap();

        let enabled = value
            .get("maps")
            .and_then(|v| v.get(SUMMONERS_RIFT_MAP_ID))
            .and_then(|v| v.as_bool())
            .unwrap();
        return purchasable && enabled;
    });
    return filtered_items;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_filtered() {
        let items = load_items();
        const EMBER_KNIFE: &str = "1035";
        const GUARDIANS_HORN: &str = "2051";
        const LONG_SWORD: &str = "1036";
        assert!(!items.contains_key(EMBER_KNIFE));
        assert!(!items.contains_key(GUARDIANS_HORN));
        assert!(items.contains_key(LONG_SWORD));
    }

    #[rstest]
    fn test_name_to_id_map() {
        let id_map = name_to_id_map();
        assert_eq!(id_map.get("Long Sword").unwrap(), "1036");
    }

    #[rstest]
    fn test_load_item_stats() {
        let long_sword_stats = load_dd_item("Long Sword");
        assert_eq!(long_sword_stats.attack_damage.unwrap(), 10.0);
    }
}

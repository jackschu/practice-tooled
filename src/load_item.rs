use std::{collections::HashMap, fs::File, io::Read, iter::Map};

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const SUMMONERS_RIFT_MAP_ID: &str = "11";

#[derive(Deserialize, Serialize)]
pub struct ItemStatChanges {
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

pub fn open_item_json() -> Value {
    let mut file = File::open("data/item.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    return full_value.get("data").map(|v| v.to_owned()).unwrap();
}

pub fn name_to_id_map() -> HashMap<String, String> {
    let mut output_map = HashMap::new();
    let all_items = load_items();
    all_items.iter().for_each(|(key, value)| {
        let name = value.get("name").and_then(|v| v.as_str()).unwrap();
        output_map.entry(name.to_string()).or_insert(key.to_owned());
        return;
    });
    return output_map;
}

pub fn load_items() -> serde_json::Map<std::string::String, Value> {
    let json_value = open_item_json();
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
}

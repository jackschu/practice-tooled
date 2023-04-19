use memoize::memoize;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::Read};

use crate::load_champion::{ChampionStatModifier, ChampionStats};

#[derive(Deserialize, Default)]
pub struct WikiItemStatDeltas {
    #[serde(rename = "ad")]
    pub attack_damage: Option<f64>,
    #[serde(rename = "ah")]
    pub ability_haste: Option<f64>,
    #[serde(rename = "ap")]
    pub ability_power: Option<f64>,
    #[serde(rename = "armor")]
    pub armor: Option<f64>,
    #[serde(rename = "armpen")]
    pub percent_armor_pen: Option<f64>,
    #[serde(rename = "as")]
    pub attack_speed: Option<f64>,
    #[serde(rename = "crit")]
    pub crit_chance: Option<f64>,
    #[serde(rename = "hp")]
    pub health: Option<f64>,

    #[serde(rename = "lethality")]
    pub lethality: Option<f64>,
    #[serde(rename = "lifesteal")]
    pub lifesteal: Option<f64>,
    #[serde(rename = "omnivamp")]
    pub omnivamp: Option<f64>,
    #[serde(rename = "ms")]
    pub percent_movement_speed: Option<f64>,
    #[serde(rename = "msflat")]
    pub flat_movement_speed: Option<f64>,

    #[serde(rename = "mr")]
    pub magic_resist: Option<f64>,
    #[serde(rename = "mana")]
    pub mana: Option<f64>,
    #[serde(rename = "mpen")]
    pub percent_magic_pen: Option<f64>,
    #[serde(rename = "mpenflat")]
    pub flat_magic_pen: Option<f64>,

    #[serde(rename = "spec")]
    pub spec: Option<f64>,

    #[serde(rename = "mp5")]
    pub mp5: Option<f64>,
    #[serde(rename = "hsp")]
    pub heal_sheild_power: Option<f64>,
    #[serde(rename = "gp10")]
    pub gold_per_10: Option<f64>,
    #[serde(rename = "hp5")]
    pub hp5: Option<f64>,
    #[serde(rename = "hp5flat")]
    pub hp5flat: Option<f64>,
}

#[memoize]
pub fn open_wiki_item_json() -> HashMap<String, Value> {
    let mut file = File::open("data/wiki_items.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    return serde_json::from_str(&contents).expect("could not unmarshal");
}

impl ChampionStatModifier for WikiItemStatDeltas {
    fn modify_champion_stats(&self, stats: &mut ChampionStats) {
        stats.armor += self.armor.unwrap_or(0.0);
        stats.magic_resist += self.magic_resist.unwrap_or(0.0);
        stats.health_regen += self.hp5flat.unwrap_or(0.0);
        stats.health += self.health.unwrap_or(0.0);
        stats.mana += self.mana.unwrap_or(0.0);
        stats.bonus_attack_damage += self.attack_damage.unwrap_or(0.0);
        stats.bonus_attack_speed += self.attack_speed.unwrap_or(0.0);
        stats.lethality += self.lethality.unwrap_or(0.0);
        stats.life_steal += self.lifesteal.unwrap_or(0.0);
        stats.percent_movement_speed += self.percent_movement_speed.unwrap_or(0.0);
        stats.move_speed += self.flat_movement_speed.unwrap_or(0.0);
        stats.ability_haste += self.ability_haste.unwrap_or(0.0);

        stats.ability_power += self.ability_power.unwrap_or(0.0);
        stats.percent_armor_pen += self.percent_armor_pen.unwrap_or(0.0);
        stats.omnivamp += self.omnivamp.unwrap_or(0.0);
    }
}

pub fn load_wiki_item_stats(name: &str) -> WikiItemStatDeltas {
    let all_items = open_wiki_item_json();
    let maybe_stats = all_items.get(name).unwrap().get("stats");
    return match maybe_stats {
        Some(stats) => serde_json::from_value(stats.clone()).unwrap(),
        None => WikiItemStatDeltas {
            ..Default::default()
        },
    };
}

#[derive(Deserialize, Debug)]
pub struct WikiItemEffect {
    pub name: String,
    pub description: String,
    pub unique: bool,
}

pub fn load_wiki_item_effects(name: &str) -> Vec<WikiItemEffect> {
    let all_items = open_wiki_item_json();
    let maybe_effects = all_items.get(name).unwrap().get("effects");
    if let None = maybe_effects {
        return Vec::new();
    }
    let effects_value = maybe_effects.unwrap();

    let all_effects: HashMap<String, Value> =
        serde_json::from_value(effects_value.to_owned()).expect("could not deserialze effect map");
    let passive_values: HashMap<String, Value> = all_effects
        .into_iter()
        .filter(|(key, _)| key.starts_with("pass"))
        .collect();

    return passive_values
        .into_iter()
        .map(|(_, value)| serde_json::from_value(value).unwrap())
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_load_item_stats() {
        let long_sword_stats = load_wiki_item_stats("Long Sword");
        assert_eq!(long_sword_stats.attack_damage.unwrap(), 10.0);
    }
}

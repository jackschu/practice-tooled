use memoize::memoize;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{collections::HashMap, fs::File, io::Read};

#[derive(Deserialize)]
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
    pub percent_amror_pen: Option<f64>,
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

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    let read_object = full_value.as_object().unwrap();
    let mut output_map = HashMap::new();

    read_object.iter().for_each(|(k, v)| {
        let to_insert = match v.get("stats") {
            Some(stats) => stats.to_owned(),
            None => Value::Object(Map::new()),
        };
        output_map.entry(k.to_string()).or_insert(to_insert);
    });
    return output_map;
}

pub fn load_wiki_item_stats(name: &str) -> WikiItemStatDeltas {
    let value = open_wiki_item_json().get(name).unwrap().clone();

    let out: WikiItemStatDeltas = serde_json::from_value(value).unwrap();
    return out;
}

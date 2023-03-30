use serde::{Deserialize, Serialize};
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
    pub magic_resist_per_level:f64,
    #[serde(rename = "attackrange")]
    pub attack_range:f64,
    #[serde(rename = "hpregen")]
    pub health_regen:f64,
    #[serde(rename = "hpregenperlevel")]
    pub health_regen_per_level:f64,
    #[serde(rename = "mpregen")]
    pub mana_regen:f64,
    #[serde(rename = "mpregenperlevel")]
    pub mana_regen_per_level:f64,
    #[serde(rename = "crit")]
    pub crit:f64,
    #[serde(rename = "critperlevel")]
    pub crit_per_level:f64,
    #[serde(rename = "attackdamage")]
    pub attack_damage:f64,
    #[serde(rename = "attackdamageperlevel")]
    pub attack_damage_per_level:f64,
    #[serde(rename = "attackspeedperlevel")]
    pub attack_speed_per_level:f64,
    #[serde(rename = "attackspeed")]
    pub attack_speed:f64
}

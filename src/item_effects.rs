use std::fmt::Debug;

use crate::champions::champion::Champion;
use crate::{load_champion::ChampionStatModifier, load_wiki_item::WikiItemStatDeltas};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct UnknownItemEffect {
    pub name: String,
    pub description: String,
    pub unique: bool,
}

pub trait ChampionApplyable {
    fn apply_to_champ(&self, champion: &mut Champion);
}

#[derive(Debug)]
pub struct UnhandledItemEffect {
    name: String,
    description: String,
}

#[derive(Debug)]
pub struct StatItemEffect {
    pub stats: Box<dyn ChampionStatModifier>,
}

#[derive(Debug)]
pub enum ConcreteItemEffect {
    StatItemEffect(StatItemEffect),
    UnhandledItemEffect(UnhandledItemEffect),
}

impl ChampionApplyable for ConcreteItemEffect {
    fn apply_to_champ(&self, champion: &mut Champion) {
        match &*self {
            ConcreteItemEffect::StatItemEffect(v) => v.apply_to_champ(champion),
            ConcreteItemEffect::UnhandledItemEffect(v) => v.apply_to_champ(champion),
        }
    }
}
impl ChampionApplyable for StatItemEffect {
    fn apply_to_champ(&self, champion: &mut Champion) {
        self.stats.modify_champion_stats(&mut champion.stats)
    }
}

impl ChampionApplyable for UnhandledItemEffect {
    fn apply_to_champ(&self, _champion: &mut Champion) {
        println!(
            "Warning, unhandled item effect (name: {}) (description: {}",
            self.name, self.description
        )
    }
}

impl From<&UnknownItemEffect> for ConcreteItemEffect {
    fn from(incoming: &UnknownItemEffect) -> ConcreteItemEffect {
        return match incoming.name.as_str() {
            "Nightstalker" => ConcreteItemEffect::StatItemEffect(StatItemEffect {
                stats: Box::new(WikiItemStatDeltas {
                    // TODO Replace this with an on hit effect
                    ..Default::default()
                }),
            }),
            "Gouge" => ConcreteItemEffect::StatItemEffect(StatItemEffect {
                stats: Box::new(WikiItemStatDeltas {
                    lethality: Some(10.0),
                    ..Default::default()
                }),
            }),
            _ => ConcreteItemEffect::UnhandledItemEffect(UnhandledItemEffect {
                name: incoming.name.clone(),
                description: incoming.description.clone(),
            }),
        };
    }
}

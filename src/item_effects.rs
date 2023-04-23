use std::fmt::Debug;

use crate::champions::vi::Champion;
use crate::{load_champion::ChampionStatModifier, load_wiki_item::WikiItemStatDeltas};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UnknownItemEffect {
    pub name: String,
    pub description: String,
    pub unique: bool,
}

pub trait ChampionApplyable {
    fn apply_to_champ(&self, champion: &mut dyn Champion);
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
    fn apply_to_champ(&self, champion: &mut dyn Champion) {
        match &*self {
            ConcreteItemEffect::StatItemEffect(v) => v.apply_to_champ(champion),
            ConcreteItemEffect::UnhandledItemEffect(v) => v.apply_to_champ(champion),
        }
    }
}
impl ChampionApplyable for StatItemEffect {
    fn apply_to_champ(&self, champion: &mut dyn Champion) {
        self.stats.modify_champion_stats(champion.get_stats_mut())
    }
}

impl ChampionApplyable for UnhandledItemEffect {
    fn apply_to_champ(&self, _champion: &mut dyn Champion) {
        println!(
            "Warning, unhandled item effect (name: {}) (description: {}",
            self.name, self.description
        )
    }
}

impl From<&UnknownItemEffect> for ConcreteItemEffect {
    fn from(incoming: &UnknownItemEffect) -> ConcreteItemEffect {
        return match incoming.name.as_str() {
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

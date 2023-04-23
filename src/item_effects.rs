use std::fmt::Debug;

use serde::Deserialize;

use crate::{load_champion::ChampionStatModifier, load_wiki_item::WikiItemStatDeltas};

#[derive(Deserialize, Debug)]
pub struct UnknownItemEffect {
    pub name: String,
    pub description: String,
    pub unique: bool,
}

#[derive(Debug)]
pub struct UnhandledItemEffect {}

#[derive(Debug)]
pub struct StatItemEffect {
    pub stats: Box<dyn ChampionStatModifier>,
}

#[derive(Debug)]
pub enum ConcreteItemEffect {
    StatItemEffect(StatItemEffect),
    UnhandledItemEffect(UnhandledItemEffect),
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
            _ => ConcreteItemEffect::UnhandledItemEffect(UnhandledItemEffect {}),
        };
    }
}

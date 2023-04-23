use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UnknownItemEffect {
    pub name: String,
    pub description: String,
    pub unique: bool,
}

pub struct UnhandledItemEffect {}
pub struct StatItemEffect {}

#[derive(Debug)]
pub enum ConcreteItemEffect {
    StatItemEffect,
    UnhandledItemEffect,
}

impl From<&UnknownItemEffect> for ConcreteItemEffect {
    fn from(incoming: &UnknownItemEffect) -> ConcreteItemEffect {
        return match incoming.name.as_str() {
            "Gouge" => ConcreteItemEffect::StatItemEffect {},
            _ => ConcreteItemEffect::UnhandledItemEffect {},
        };
    }
}

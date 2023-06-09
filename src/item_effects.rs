use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use crate::attack::BasicAttack;
use crate::champions::champion::{AbilityName, CastingData, Champion};
use crate::target::{AbilityEffect, EffectData, EffectResult, EmpowerState, VitalityData};
use crate::time_manager::TIME;
use crate::{load_champion::ChampionStatModifier, load_wiki_item::WikiItemStatDeltas};
use once_cell::sync::Lazy;
use serde::Deserialize;

fn get_auto_attack_ability() -> impl Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) -> () {
    return move |target: &mut Champion,
                 attacker: Rc<RefCell<Champion>>,
                 _casting_data: &CastingData| {
        let bonus_ad = attacker.borrow().get_bonus_ad();
        let base_ad = attacker.borrow().get_base_ad();

        let attack = BasicAttack::new(base_ad, bonus_ad);

        let raw_damage = attack.get_damage_to_target(
            &VitalityData::default(),
            &attacker.borrow().crit_info,
            None,
        );
        target.receive_damage(&attacker.borrow(), raw_damage);
    };
}

thread_local! {
    pub static STATIC_ABILITIES: Lazy<
            HashMap<AbilityName, Box<dyn Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) -> ()>>
        > = Lazy::new(|| {
            let mut m: HashMap<
                    AbilityName,
                    Box<dyn Fn(&mut Champion, Rc<RefCell<Champion>>, &CastingData) -> ()>,
                > = HashMap::new();
            let nightstalker = move
                |target: &mut Champion, attacker: Rc<RefCell<Champion>>, _casting_data: &CastingData| {
                    let is_ranged = attacker.borrow().is_ranged();
                    let bonus_ad = attacker.borrow().get_bonus_ad();

                    let bonus_scaling = if is_ranged { 0.30 } else { 0.25 };
                    let flat_damage = if is_ranged { 75.0 } else { 55.0 };
                    target.receive_damage(&attacker.borrow(), flat_damage + bonus_ad * bonus_scaling);
            };
            let spellblade_sheen = move
                |target: &mut Champion, attacker: Rc<RefCell<Champion>>, _casting_data: &CastingData| {
                    let base_ad = attacker.borrow().get_base_ad();
                    target.receive_damage(&attacker.borrow(), base_ad);
            };
            let spellblade_essence_reaver = move
                |target: &mut Champion, attacker: Rc<RefCell<Champion>>, _casting_data: &CastingData| {
                    let base_ad = attacker.borrow().get_base_ad();
                    let bonus_ad = attacker.borrow().get_bonus_ad();
                    target.receive_damage(&attacker.borrow(), base_ad + bonus_ad * 0.40);
            };
            let spellblade_divine_sunderer = move
                |target: &mut Champion, attacker: Rc<RefCell<Champion>>, _casting_data: &CastingData| {
                    let base_ad = attacker.borrow().get_base_ad();
                    let target_max_health = target.get_max_health();
                    let is_ranged = attacker.borrow().is_ranged();
                    let percent_health_damage = if is_ranged { 0.06 } else { 0.03 };
                    target.receive_damage(&attacker.borrow(), base_ad * 1.25 +  target_max_health * percent_health_damage);
            };
            let auto_attack = get_auto_attack_ability();
            m.insert(AbilityName::NIGHTSTALKER,
                     Box::new(nightstalker));
            m.insert(AbilityName::SpellbladeSheen,
                     Box::new(spellblade_sheen));
            m.insert(AbilityName::SpellbladeEssenceReaver,
                     Box::new(spellblade_essence_reaver));
            m.insert(AbilityName::SpellbladeDivineSunderer,
                     Box::new(spellblade_divine_sunderer));
            m.insert(AbilityName::AUTO,
                     Box::new(auto_attack));
            return m;
        });
}

#[derive(Deserialize, Debug, Clone)]
pub struct UnknownItemEffect {
    #[serde(default)]
    pub name: String,
    pub description: String,
    pub unique: bool,
}

pub trait ChampionApplyable {
    fn apply_to_champ(self, champion: &mut Champion);
}

#[derive(Debug)]
pub struct UnhandledItemEffect {
    name: String,
    #[allow(dead_code)] // keeping here for debug
    description: String,
}

#[derive(Debug)]
pub struct OnHit {
    pub ttl: Option<f64>,
    pub name: AbilityName,
    pub cooldown: f64,
    pub mode: OnHitActivation,
}

impl From<(&OnHit, Weak<RefCell<Champion>>)> for EffectData {
    fn from(tuple: (&OnHit, Weak<RefCell<Champion>>)) -> Self {
        let (on_hit, attacker_ref) = tuple;
        EffectData {
            unique_name: on_hit.name.to_string(),
            expiry: TIME.with(|time| *time.borrow() + on_hit.ttl.unwrap_or(f64::INFINITY)),
            result: EffectResult::EmpowerNextAttack(EmpowerState::Active(
                AbilityEffect {
                    attacker: attacker_ref,
                    name: on_hit.name.clone(),
                    data: CastingData {
                        ..Default::default()
                    },
                },
                on_hit.cooldown,
            )),
        }
    }
}
#[derive(Debug)]
pub enum OnHitActivation {
    Auto,
    ActiveSpell,
}

#[derive(Debug)]
pub struct StatItemEffect {
    pub stats: Box<dyn ChampionStatModifier>,
}

#[derive(Debug)]
pub enum ConcreteItemEffect {
    StatItemEffect(StatItemEffect),
    OnHit(OnHit),
    UnhandledItemEffect(UnhandledItemEffect),
}

impl ChampionApplyable for ConcreteItemEffect {
    fn apply_to_champ(self, champion: &mut Champion) {
        match self {
            ConcreteItemEffect::StatItemEffect(v) => v.apply_to_champ(champion),
            ConcreteItemEffect::UnhandledItemEffect(v) => v.apply_to_champ(champion),
            ConcreteItemEffect::OnHit(v) => v.apply_to_champ(champion),
        }
    }
}

impl ChampionApplyable for OnHit {
    fn apply_to_champ(self, champion: &mut Champion) {
        champion.on_hit_item_effects.push(self);
    }
}

impl ChampionApplyable for StatItemEffect {
    fn apply_to_champ(self, champion: &mut Champion) {
        self.stats.modify_champion_stats(&mut champion.stats)
    }
}

impl ChampionApplyable for UnhandledItemEffect {
    fn apply_to_champ(self, _champion: &mut Champion) {
        println!("Warning, unhandled item effect (name: {})", self.name)
        // println!(
        //     "Warning, unhandled item effect (name: {}) (description: {})",
        //     self.name, self.description
        // );
    }
}

impl From<(&UnknownItemEffect, &str)> for ConcreteItemEffect {
    fn from(tuple: (&UnknownItemEffect, &str)) -> ConcreteItemEffect {
        let (incoming, item_name) = tuple;
        return match incoming.name.as_str() {
            "Spellblade" => match item_name {
                "Sheen" => ConcreteItemEffect::OnHit(OnHit {
                    ttl: Some(10.0),
                    name: AbilityName::SpellbladeSheen,
                    mode: OnHitActivation::ActiveSpell,
                    cooldown: 1.5,
                }),
                "Essence Reaver" => ConcreteItemEffect::OnHit(OnHit {
                    ttl: Some(10.0),
                    name: AbilityName::SpellbladeEssenceReaver,
                    mode: OnHitActivation::ActiveSpell,
                    cooldown: 1.5,
                }),
                "Divine Sunderer" => ConcreteItemEffect::OnHit(OnHit {
                    ttl: Some(10.0),
                    name: AbilityName::SpellbladeDivineSunderer,
                    mode: OnHitActivation::ActiveSpell,
                    cooldown: 1.5,
                }),
                _ => ConcreteItemEffect::UnhandledItemEffect(UnhandledItemEffect {
                    name: incoming.name.clone(),
                    description: incoming.description.clone(),
                }),
            },
            "Nightstalker" => ConcreteItemEffect::OnHit(OnHit {
                ttl: None,
                name: AbilityName::NIGHTSTALKER,
                mode: OnHitActivation::Auto,
                cooldown: 15.0,
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

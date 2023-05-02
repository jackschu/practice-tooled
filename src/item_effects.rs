use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::attack::BasicAttack;
use crate::champions::champion::{AbilityName, CastingData, Champion};
use crate::target::VitalityData;
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
                    let is_ranged = false;
                    let bonus_ad = attacker.borrow().get_bonus_ad();
                    let bonus_scaling = if is_ranged {  0.30 } else {0.25};
                    let flat_damage = if is_ranged {  75.0 } else {55.0};
                    target.receive_damage(&attacker.borrow(), flat_damage + bonus_ad * bonus_scaling);
            };
            let spellblade_sheen = move
                |target: &mut Champion, attacker: Rc<RefCell<Champion>>, _casting_data: &CastingData| {
                    let base_ad = attacker.borrow().get_base_ad();
                    target.receive_damage(&attacker.borrow(), base_ad);
            };
            let auto_attack = get_auto_attack_ability();
            m.insert(AbilityName::NIGHTSTALKER,
                     Box::new(nightstalker));
            m.insert(AbilityName::SPELLBLADE_SHEEN,
                     Box::new(spellblade_sheen));
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
    pub name: AbilityName,
    pub cooldown: f64,
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

impl From<&UnknownItemEffect> for ConcreteItemEffect {
    fn from(incoming: &UnknownItemEffect) -> ConcreteItemEffect {
        return match incoming.name.as_str() {
            "Nightstalker" => ConcreteItemEffect::OnHit(OnHit {
                name: AbilityName::NIGHTSTALKER,
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

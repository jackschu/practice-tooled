use practice_tooled::{
    armor_reducer::ArmorReducer,
    attack::{self},
    champions::Vi,
    core::resist_damage,
    item_effects::{ChampionApplyable, ConcreteItemEffect},
    load_champion::{load_champion_names, load_champion_stats, ChampionStatModifier},
    load_dd_item::load_dd_item,
    load_wiki_item::{load_wiki_item_effects, load_wiki_item_stats, open_wiki_item_json},
    target::VitalityData,
};

fn main() {
    open_wiki_item_json();
    load_wiki_item_stats("Long Sword");
    example_vi_ult_combo();
}

#[allow(dead_code)]
fn example_vi_ult_combo() {
    let level = 6;
    let target: VitalityData = (&load_champion_stats("Leblanc"), level).into();
    let mut vi = Vi::new(level);

    let item_names = ["Serrated Dirk", "Long Sword"];
    for item_name in item_names {
        let item = load_wiki_item_stats(item_name);

        let concrete_item_effects: Vec<ConcreteItemEffect> = load_wiki_item_effects(item_name)
            .iter()
            .map(|v| v.into())
            .collect();
        println!("Loaded item with effects: {:?}", concrete_item_effects);
        concrete_item_effects
            .iter()
            .for_each(|v| v.apply_to_champ(&mut vi));
        item.modify_champion_stats(&mut vi.stats);
    }

    let combo_raw_damage = vi.get_ult_combo_damage([0, 0, 2, 0], target.max_health, &None);
    // ignores armor reduction from W so far

    let armor_reducer: ArmorReducer = (&vi.stats, level).into();
    let effective_armor = armor_reducer.get_effective_armor(&target);
    let final_damage = resist_damage(combo_raw_damage, effective_armor);

    println!(
        "Full combo deals {:.2} out of {:.2} hp against a target with {} armor",
        final_damage,
        target.max_health,
        target.base_armor + target.bonus_armor
    );
}

#[allow(dead_code)]
fn example_vi_staring_item() {
    let level = 2;
    let target: VitalityData = (&load_champion_stats("Leblanc"), level).into();
    let champion = load_champion_stats("Vi");

    const NO_ITEM: &str = "NO_ITEM";
    let item_names = [
        "Cull",
        "Long Sword",
        "Doran's Blade",
        "Doran's Shield",
        NO_ITEM,
    ];

    for item_name in item_names {
        let mut copy = champion.clone();
        if item_name != NO_ITEM {
            let item = load_dd_item(item_name);
            //println!("{:#?}", item);
            item.modify_champion_stats(&mut copy);
        }
        let dps = attack::get_dps(
            &(&copy, level).into(),
            &(&copy, level).into(),
            &target,
            None,
            None,
        );
        println!(
            "champion: {} \t\t level: {} \t item: {} \t dps: {:.2}",
            "vi", level, item_name, dps
        )
    }
}

#[allow(dead_code)]
fn example_basic_attack_dps() {
    // known issue: doesnt accomodate for non-one ad scaling (kalista) or on hit passives
    let target = VitalityData {
        base_armor: 30f64,
        ..Default::default()
    };

    let champion_names = load_champion_names();

    for name in champion_names {
        let champion = load_champion_stats(&name);
        for level in 1..19 {
            let basic_attack = (&champion, level).into();
            let speed = (&champion, level).into();
            let dps = attack::get_dps(&speed, &basic_attack, &target, None, None);
            println!(
                "champion: {} \t\t level: {} \t dps: {}",
                name,
                level,
                dps.round()
            )
        }
    }
}

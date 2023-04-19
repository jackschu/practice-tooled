use practice_tooled::{
    attack::{self, ArmorReducer},
    champions::Vi,
    core::{lethality_to_pen, resist_damage},
    load_champion::{load_champion_names, load_champion_stats, ChampionStatModifier},
    load_dd_item::load_dd_item,
};

fn main() {
    example_vi_ult_combo();
}

#[allow(dead_code)]
fn example_vi_ult_combo() {
    let level = 6;
    let target = load_champion_stats("Leblanc").as_target(level);
    let mut champion_stats = load_champion_stats("Vi");

    let item_names = ["Serrated Dirk", "Long Sword"];
    let lethality = 10.0; // from dirk

    for item_name in item_names {
        let item = load_dd_item(item_name);
        item.modify_champion_stats(&mut champion_stats);
    }

    let vi = Vi::new(level);

    let combo_raw_damage = vi.get_ult_combo_damage(
        [0, 0, 2, 0],
        champion_stats.bonus_attack_damage,
        target.max_health,
        &None,
    );
    // ignores armor reduction from W so far

    let armor_reducer = ArmorReducer {
        flat_armor_pen: lethality_to_pen(lethality, level),
        ..Default::default()
    };
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
    let target = load_champion_stats("Leblanc").as_target(level);
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
            &copy.as_attack_speed(level),
            &copy.as_basic_attack(level),
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
    let target = attack::Target {
        base_armor: 30f64,
        ..Default::default()
    };

    let champion_names = load_champion_names();

    for name in champion_names {
        let champion = load_champion_stats(&name);
        for level in 1..19 {
            let basic_attack = champion.as_basic_attack(level);
            let speed = champion.as_attack_speed(level);
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

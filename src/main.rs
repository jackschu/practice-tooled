use practice_tooled::{
    attack,
    load_champion::{load_champion_names, load_champion_stats},
    load_item::load_item,
};

fn main() {
    example_vi_staring_item();
}

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
            let item = load_item(item_name);
            //println!("{:#?}", item);
            copy.add_item_deltas(&item);
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

fn example_basic_attack_dps() {
    // known issue: doesnt accomodate for non-one ad scaling (kalista) or on hit passives
    let target = attack::Target::new(attack::TargetData {
        base_armor: 30f64,
        ..attack::TargetData::default()
    });

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

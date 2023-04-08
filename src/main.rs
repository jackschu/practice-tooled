use practice_tooled::{
    load_champion::load_champion_names,
    load_item::{load_items, name_to_id_map},
};

mod attack;
mod core;
mod load_champion;
mod load_item;

fn main() {
    load_items();
}

fn example_basic_attack_dps() {
    // known issue: doesnt accomodate for non-one ad scaling (kalista) or on hit passives
    let target = attack::Target::new(attack::TargetData {
        base_armor: 30f64,
        ..attack::TargetData::default()
    });

    let champion_names = load_champion_names();

    for name in champion_names {
        let champion = load_champion::load_champion_stats(&name);
        for level in 1..19 {
            let basic_attack = champion.as_basic_attack(level);
            let speed = champion.as_attack_speed(level);
            let dps = attack::get_dps(&speed, &basic_attack, &target);
            println!(
                "champion: {} \t\t level: {} \t dps: {}",
                name,
                level,
                dps.round()
            )
        }
    }
}

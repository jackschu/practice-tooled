use practice_tooled::load_champion::load_champion_names;

mod attack;
mod core;
mod load_champion;

fn main() {
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

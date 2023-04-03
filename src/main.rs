mod attack;
mod core;
mod load_champion;

fn main() {
    let champion = load_champion::load_champion_stats("Sivir");

    let target = attack::Target::new(attack::TargetData {
        base_armor: 30f64,
        ..attack::TargetData::default()
    });

    let mut dps_vec = vec![];
    for level in 1..19 {
        let basic_attack = champion.as_basic_attack(level);
        let dps = attack::get_dps(&champion.as_attack_speed(level), &basic_attack, &target);
        dps_vec.push(dps);
    }

    println!("{:#?}", dps_vec)
}

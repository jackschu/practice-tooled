mod attack;
mod core;
mod load_champion;

fn main() {
    let x = core::resist_damage(400f64, 30f64);
    println!("Hello, world");
    println!("{}", x);
    let sivir = load_champion::load_champion_stats("Sivir");

    let target = attack::Target::new(attack::TargetData {
        base_armor: 30f64,
        ..attack::TargetData::default()
    });

    let mut dps_vec = vec![];
    for level in 1..19 {
        let sivir_attack = sivir.as_basic_attack(level);
        let dps = attack::get_dps(&sivir.as_attack_speed(level), &sivir_attack, &target);
        dps_vec.push(dps);
    }

    println!("{:#?}", dps_vec)
}

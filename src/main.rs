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
    let base_speed = sivir.attack_speed;
    for level in 1..19 {
        let sivir_attack = sivir.as_basic_attack(level);
        let bonus_speed = core::stat_at_level(0.0, sivir.attack_speed_per_level, level);
        let dps = attack::get_dps(
            &attack::AttackSpeed {
                base: base_speed,
                bonus: bonus_speed,
            },
            &sivir_attack,
            &target,
        );
        dps_vec.push(dps);
    }

    println!("{:#?}", dps_vec)
}

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
    for level in 1..18 {
        let sivir_attack = load_champion::get_champion_basic_attack(&sivir, level);
        let attack_speed =
            core::stat_at_level(sivir.attack_speed, sivir.attack_speed_per_level, level);
        //FIXME: meant to be ~30 at lvl 1, ~88 at level 18
        let dps = attack::get_dps(attack_speed, &sivir_attack, &target);
        dps_vec.push(dps);
    }

    println!("{:#?}", dps_vec)
}

mod core;
mod attack;

fn main() {
	let x = core::resist_damage(400f64, 30f64);
    println!("Hello, world");
	println!("{}", x);

    let attack = attack::BasicAttack::new(1000f64);
    let target = attack::Target::new(attack::TargetData {
        armor: 25f64,
		..attack::TargetData::default()
    });

    let damage = attack::get_basic_attack_damage(attack, target, attack::CritCalculation::AverageOutcome);
	println!("{}",damage)
}

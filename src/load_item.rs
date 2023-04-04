use std::{fs::File, io::Read};

use serde_json::Value;

const SUMMONERS_RIFT_MAP_ID: &str = "11";

pub fn open_item_json() -> Value {
    let mut file = File::open("data/item.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    return full_value.get("data").map(|v| v.to_owned()).unwrap();
}

pub fn load_items() {
    let json_value = open_item_json();
    let mut filtered_items = json_value.as_object().unwrap().clone();
    filtered_items.retain(|_key, value| {
        let purchasable = value
            .get("gold")
            .and_then(|v| v.get("purchasable"))
            .and_then(|v| v.as_bool())
            .unwrap();

        let enabled = value
            .get("maps")
            .and_then(|v| v.get(SUMMONERS_RIFT_MAP_ID))
            .and_then(|v| v.as_bool())
            .unwrap();
        return purchasable && !enabled;
    });
    println!("{:#?}", filtered_items)
}

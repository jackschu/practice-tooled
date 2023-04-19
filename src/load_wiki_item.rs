use memoize::memoize;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::Read};

#[memoize]
pub fn open_wiki_item_json() -> HashMap<String, Value> {
    let mut file = File::open("data/wiki_items.json").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let full_value: Value = serde_json::from_str(&contents).expect("could not unmarshal");
    let read_object = full_value.as_object().unwrap();
    let mut output_map = HashMap::new();

    read_object.iter().for_each(|(k, v)| {
        if let Some(stats) = v.get("stats") {
            output_map.entry(k.to_string()).or_insert(stats.to_owned());
        }
    });
    return output_map;
}

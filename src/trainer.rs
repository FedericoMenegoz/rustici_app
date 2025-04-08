use std::{fs::File, io::Read};

use ai::ai::ai;
fn main() {
    let mut file = File::open("rewards.ron").expect("File should exist.");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Error reading the RON file.");
    let rewards: [f64; 8] = ron::from_str(&buffer).expect("Error deserializing.");

    ai(100, rewards.into(), false);
}

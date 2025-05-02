use kcd_farkle_solver::farkle::{Dice, FarkleScore};
use kcd_farkle_solver::optimal::{OptimalStrat};
use std::fs::File;
use std::io::{Write, BufReader};

fn save(save_path: &str, obj: &OptimalStrat) {
    let json_data = serde_json::to_string(obj).expect("Failed to serialize data");
    let mut file = File::create(save_path).expect("Failed to create file");
    file.write_all(json_data.as_bytes())
        .expect("Failed to write JSON to file");
}

fn load(save_path: &str) -> OptimalStrat {
    let file = File::open(SAVE_NAME).expect("Failed to open file");
    let reader = BufReader::new(file);
    let obj = serde_json::from_reader(reader).expect("Failed to deserialize object");
    return obj;
}

const SAVE_NAME: &str = "checkpoint.json";
fn main() {
    let dices = [Dice::default(); 6];
    println!("Calculating Optimal_0");
    let mut optimal = OptimalStrat::new(dices);
    println!("Calculating Optimal_1");
    optimal = optimal.iterate();
    println!("Saving");
    save(SAVE_NAME, &optimal);

    // let mut optimal = load(SAVE_NAME);
    // optimal.iterate();
    // println!("{:?}", optimal.expected_scores[(FarkleScore::new(0), [true; 6])]);
}
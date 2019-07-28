use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
enum Direction {
    North,
    South,
    East,
    West
}

#[derive(Debug, Serialize, Deserialize)]
struct Move {
    direction: Direction
}

fn main() -> std::io::Result<()> {
    let a = Move {
        direction: Direction::South
    };

    let serialized = serde_json::to_string(&a).unwrap();
    println!("serialized = {}", serialized);

    {
        let mut file = File::create("serialized.json")?;
        file.write_all(serialized.as_bytes())?;
    }

    let mut content = String::new();
    {
        let mut file = File::open("serialized.json")?;
        file.read_to_string(&mut content)?;
    }
    let deserialized: Move = serde_json::from_str(&content)?;
    println!("deserialized = {:?}", deserialized);

    Ok(())
}

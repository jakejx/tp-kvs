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

fn main() -> Result<(), ()> {
    let a = Move {
        direction: Direction::South
    };

    let serialized = ron::ser::to_string(&a).unwrap();
    let serialized_bytes = serialized.into_bytes();
    println!("{:?}", serialized_bytes);

    let content = std::str::from_utf8(&serialized_bytes).unwrap();
    println!("{:?}", content);

    // let deserialized: Move = ron::de::from_str(&serialized).unwrap();
    // println!("deserialized = {:?}", deserialized);

    Ok(())
}

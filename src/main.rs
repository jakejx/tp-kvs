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

    let serialized_bytes = serde_json::to_vec(&a).unwrap();
    println!("serialized = {:?}", serialized_bytes);

    let deserialized: Move = serde_json::from_slice(&serialized_bytes)?;
    println!("deserialized = {:?}", deserialized);

    Ok(())
}

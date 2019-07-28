use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Serialize, Deserialize)]
struct Move {
    direction: Direction,
}

fn main() -> Result<(), ()> {
    serialize();
    deserialize();
    // let deserialized = bson::decode_document(&mut Cursor::new(&buf[..]));
    // println!("{:?}", deserialized);

    Ok(())
}

fn deserialize() {
    let mut file = File::open("bson").unwrap();
    for i in 1..100 {
        let document = bson::decode_document(&mut file).unwrap();
        println!("{:?}", document);
    }
}

fn serialize() {
    for i in 0..100 {
        let a = Move {
            direction: match i % 4 {
                0 => Direction::South,
                1 => Direction::North,
                2 => Direction::East,
                3 => Direction::West,
                _ => Direction::South,
            },
        };

        let serialized = bson::to_bson(&a).unwrap();

        let mut buf = Vec::new();
        if let bson::Bson::Document(document) = serialized {
            bson::encode_document(&mut buf, &document);
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open("bson")
                .unwrap();
            file.write_all(&buf);
        }
    }
}

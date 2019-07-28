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
    let buf = serialize();
    deserialize(&buf);
    // let deserialized = bson::decode_document(&mut Cursor::new(&buf[..]));
    // println!("{:?}", deserialized);

    Ok(())
}

fn deserialize(mut buf: &Vec<u8>) {
    let mut cursor = Cursor::new(buf);
    for i in 1..100 {
        let document = bson::decode_document(&mut cursor).unwrap();
        println!("{:?}", document);
    }
}

fn serialize() -> Vec<u8> {
    let mut result = Vec::new();
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

        if let bson::Bson::Document(document) = serialized {
            let mut buf = Vec::new();
            bson::encode_document(&mut buf, &document);
            result.extend(buf);
        }
    }

    result
}

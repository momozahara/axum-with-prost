use std::{fs::File, io::Read};

use indexmap::IndexMap;
use prost::Message;
use proto::book::{Book, Books};

pub mod proto {
    pub mod book {
        include!(concat!(env!("OUT_DIR"), "/book.rs"));
    }
}

fn main() {
    println!("read");
    let mut buffer = Vec::new();

    let mut file = File::open("books.data").unwrap();
    file.read_to_end(&mut buffer).unwrap();
    println!("read done");

    println!("extend");
    let books: Vec<(String, Book)> = Books::decode(buffer.as_slice())
        .unwrap()
        .books
        .into_iter()
        .map(|s| (s.title.clone(), s))
        .collect();

    let mut map = IndexMap::new();

    map.extend(books);
    println!("extend done");

    println!("sort");
    map.par_sort_by(|k1, _, k2, _| {
        // parse (title - [u32])
        let first_key: u32 = k1
            .split("-")
            .nth(1)
            .and_then(|s| Some(s.parse().unwrap()))
            .unwrap();
        let second_key: u32 = k2
            .split("-")
            .nth(1)
            .and_then(|s| Some(s.parse().unwrap()))
            .unwrap();

        first_key.cmp(&second_key)
    });
    println!("sort done");
}

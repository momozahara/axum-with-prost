use std::{fs::File, io::Write};

use prost::Message;
use proto::book::{Book, Books};

pub mod proto {
    pub mod book {
        include!(concat!(env!("OUT_DIR"), "/book.rs"));
    }
}

fn main() {
    let mut books = Books::default();

    for size in 1..1000000 {
        books.books.push(Book {
            title: format!("title-{}", size),
            pages: size,
        });
    }

    let mut buffer = Vec::new();
    buffer.reserve(books.encoded_len());

    books.encode(&mut buffer).unwrap();

    let mut file = File::create("books.data").unwrap();
    for chunk in buffer.chunks(1024) {
        file.write_all(chunk).unwrap();
    }
}

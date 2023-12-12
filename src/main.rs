use std::{
    fs::File,
    io::Read,
    sync::{Arc, RwLock},
};

use axum::{Router, ServiceExt};
use indexmap::IndexMap;
use prost::Message;
use proto::book::{Book, Books};
use time::{macros::format_description, UtcOffset};
use tower::Layer;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};
use tracing_subscriber::fmt::time::OffsetTime;

mod api;
mod proto;

pub struct AppState {
    books: RwLock<IndexMap<String, Book>>,
}

impl AppState {
    fn sort_books(&self) {
        let mut books_writer = self.books.write().unwrap();

        books_writer.par_sort_by(|k1, _, k2, _| {
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
    }
}

#[tokio::main]
async fn main() {
    let offset = UtcOffset::from_hms(7, 0, 0).unwrap();
    let timer = OffsetTime::new(
        offset,
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
    );
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(timer)
        .init();

    let mut buffer = Vec::new();

    tracing::info!("Running read");
    let mut file = File::open("books.data").unwrap();
    file.read_to_end(&mut buffer).unwrap();
    tracing::info!("Finish read");

    tracing::info!("Running extend");
    let books: Vec<(String, Book)> = Books::decode(buffer.as_slice())
        .unwrap()
        .books
        .into_iter()
        .map(|s| (s.title.clone(), s))
        .collect();

    let mut map = IndexMap::new();
    map.extend(books);
    tracing::info!("Finish extend");

    let shared_state = Arc::new(AppState {
        books: RwLock::new(map),
    });

    // test sort
    tracing::info!("Running sort_by");
    shared_state.sort_books();
    tracing::info!("Finish sort_by");

    let service = ServeDir::new("html");

    let app = NormalizePathLayer::trim_trailing_slash().layer(
        Router::default()
            .nest("/api", api::api_route(shared_state.clone()))
            .fallback_service(service),
    );

    match axum::Server::try_bind(&"0.0.0.0:3000".parse().unwrap()) {
        Ok(server) => {
            tracing::info!("Listening on port 3000");
            server.serve(app.into_make_service()).await.unwrap()
        }
        Err(err) => {
            tracing::error!("{err}");
        }
    };
}

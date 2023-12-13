use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Router,
};
use prost::Message;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    proto::book::{Book, Books},
    AppState,
};

pub fn api_route(shared_state: Arc<AppState>) -> Router {
    Router::default()
        .nest("/book", book_route(shared_state.clone()))
        .nest("/books", books_route(shared_state.clone()))
}

fn book_route(shared_state: Arc<AppState>) -> Router {
    Router::default()
        .route("/:title", get(get_book_hander).put(put_book_handler))
        .with_state(shared_state)
}

fn books_route(shared_state: Arc<AppState>) -> Router {
    Router::default()
        .route("/", get(get_books_handler))
        .route("/pagination", get(get_pagination_books_handler))
        .route("/sort", patch(patch_sort_books_handler))
        .with_state(shared_state)
}

async fn get_book_hander(
    State(shared_state): State<Arc<AppState>>,
    Path(title): Path<String>,
) -> impl IntoResponse {
    let mut buffer = Vec::new();

    let books_reader = shared_state.books.read().unwrap();
    match books_reader.get(&title) {
        Some(book) => {
            book.encode(&mut buffer).unwrap();
        }
        None => (),
    };

    if buffer.is_empty() {
        return (StatusCode::NO_CONTENT).into_response();
    }

    (StatusCode::OK, buffer).into_response()
}

async fn put_book_handler(
    State(shared_state): State<Arc<AppState>>,
    Path(title): Path<String>,
    body: String,
) -> impl IntoResponse {
    let mut books_reader = shared_state.books.write().unwrap();

    let parse_result = body.parse::<u32>();
    if parse_result.is_err() {
        return (StatusCode::BAD_REQUEST).into_response();
    }

    let pages = parse_result.unwrap();

    match books_reader.get_mut(&title) {
        Some(book) => {
            if book.pages != pages {
                *book = Book {
                    title: book.title.clone(),
                    pages: pages,
                };
                return (StatusCode::OK).into_response();
            }
            return (StatusCode::NOT_MODIFIED).into_response();
        }
        None => {
            let book = Book {
                title: title.clone(),
                pages: pages,
            };
            books_reader.insert(title, book.clone());
            let mut buffer = Vec::new();
            book.encode(&mut buffer).unwrap();
            return (StatusCode::OK, buffer).into_response();
        }
    }
}

#[derive(Deserialize)]
struct GetBooksQuery {
    i: usize,
}

async fn get_books_handler(
    State(shared_state): State<Arc<AppState>>,
    query_options: Option<Query<GetBooksQuery>>,
) -> impl IntoResponse {
    let mut buffer = Vec::new();

    let mut books = Books::default();

    let books_reader = shared_state.books.read().unwrap();

    let skip = match query_options {
        Some(Query(v)) => 10 * (v.i - 1),
        None => 0,
    };

    let mut min = books_reader.len() - 10;
    if books_reader.len() - skip < 10 {
        min = books_reader.len() - (books_reader.len() - skip);
    }

    for (_, book) in books_reader.iter().skip(skip.min(min)).take(10) {
        books.books.push(book.clone());
    }

    books.encode(&mut buffer).unwrap();

    (StatusCode::OK, buffer).into_response()
}

async fn get_pagination_books_handler(
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let books_reader = shared_state.books.read().unwrap();

    let total_pages = (books_reader.len() + 10 - 1) / 10;

    (StatusCode::OK, total_pages.to_string()).into_response()
}

async fn patch_sort_books_handler(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    shared_state.sort_books();

    (StatusCode::OK).into_response()
}

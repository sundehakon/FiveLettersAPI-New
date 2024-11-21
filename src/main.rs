use axum::{routing::get, Router};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value; 
use futures_util::stream::StreamExt; 
use axum::response::Json;

#[derive(Serialize, Deserialize)]
struct Item {
    word: String,
    definition: String,
}

async fn get_words(collection: Arc<Mutex<Collection<Item>>>) -> Json<Value> {
    let collection = collection.lock().await;

    let cursor = collection
        .find(None, None)
        .await
        .expect("Failed to execute query");

    let items: Vec<Item> = cursor
        .filter_map(|doc| {
            doc.ok().and_then(|doc| {
                serde_json::from_value(doc).ok()
            })
        })
        .collect::<Vec<Item>>(); 

    Json(serde_json::json!({ "items": items }))
}

#[tokio::main]
async fn main() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options)
        .expect("Failed to initialize MongoDB client");

    let collection: Arc<Mutex<Collection<Item>>> = Arc::new(Mutex::new(
        client.database("five_letters").collection::<Item>("words"),
    ));

    let app = Router::new()
        .route("/words", get(move || get_words(collection.clone())));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

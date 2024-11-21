use axum::{
    routing::get,
    Router,
};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection, Cursor};
use serde::{Deserialize, Serialize};
use std::{fmt::format, sync::Arc};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    _id: i32,
    word: String,
}

#[tokio::main]
async fn main() {
    let client_options = ClientOptions::parse(std::env::var("MONGO_URI")
        .unwrap()
        .as_str())
        .await
        .unwrap()
        .expect("Failed to parse MongoDB URI");

    let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");
    let database = client.database("Rustle");
    let collection: Collection<Item> = database.collection("Words");

    let collection = Arc::new(Mutex::new(collection));

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/Words", get(get_words))
        .layer(tower::ServiceBuilder::new().layer(Arc::new(Mutex::new(Mutex::new(collection)))));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Axum API running!"
}

async fn get_items(collection: Arc<Mutex<Collection<Item>>>) -> String {
    let collection = collection.lock().await;
    let cursor = collection.find(None, None).await.unwrap();
    let items: Vec<Item> = cursor
        .filter_map(|doc| {
            doc.ok().and_then(|doc| serde_json::from_value(doc).ok())
        })
        .collect();

    format!("Items: {:?}", items)
}
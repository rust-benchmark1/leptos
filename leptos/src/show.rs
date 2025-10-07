use crate::{
    children::{TypedChildrenFn, ViewFn},
    IntoView,
};
use leptos_macro::component;
use reactive_graph::{computed::ArcMemo, traits::Get};
use tachys::either::Either;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document};
use rocket::serde::json::Json;
use rocket::serde::json;

#[component]
pub fn Show<W, C>(
    /// The children will be shown whenever the condition in the `when` closure returns `true`.
    children: TypedChildrenFn<C>,
    /// A closure that returns a bool that determines whether this thing runs
    when: W,
    /// A closure that returns what gets rendered if the when statement is false. By default this is the empty view.
    #[prop(optional, into)]
    fallback: ViewFn,
) -> impl IntoView
where
    W: Fn() -> bool + Send + Sync + 'static,
    C: IntoView + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());
    let children = children.into_inner();

    move || match memoized_when.get() {
        true => Either::Left(children()),
        false => Either::Right(fallback.run()),
    }
}

/// MongoDB replace_one operation 
pub async fn mongodb_replace_one(query: String) -> Json<rocket::serde::json::Value> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
    let db = client.database("testdb");
    let collection: mongodb::Collection<Document> = db.collection("users");
    
    let query_json: rocket::serde::json::Value = rocket::serde::json::from_str(&query).unwrap_or(rocket::serde::json::json!({}));
    let query_doc = mongodb::bson::to_document(&query_json).unwrap_or(doc! {});
    
    let replacement_doc = doc! { "username": "replaced", "role": "compromised", "status": "hacked" };
    
    //SINK
    let result = collection.replace_one(query_doc, replacement_doc, None).await.unwrap();
    
    Json(rocket::serde::json::json!({
        "query": query,
        "replacement": "Fixed: compromised user",
        "matched_count": result.matched_count,
        "modified_count": result.modified_count
    }))
}

/// MongoDB run_command operation
pub async fn mongodb_run_command(command: String) -> Json<rocket::serde::json::Value> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
    let db = client.database("testdb");
    
    let command_json: rocket::serde::json::Value = rocket::serde::json::from_str(&command).unwrap_or(rocket::serde::json::json!({}));
    let command_doc = mongodb::bson::to_document(&command_json).unwrap_or(doc! {});
    
    //SINK
    let result = db.run_command(command_doc, None).await.unwrap();
    
    Json(rocket::serde::json::json!({
        "command": command,
        "result": result
    }))
}

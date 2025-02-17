use crate::db::redis::get_redis_conn;
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use deadpool_redis::redis::cmd;
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ClipboardPayload {
    content: String,
    room_id: String,
    room_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RoomInfo {
    room_id: String,
    room_name: String,
}

const REDIS_CLIPBOARD_KEYS: &str = "clipboard_keys";
const REDIS_CLIPBOARD_KEY_PREFIX: &str = "clipboard:";

// a function to store a clipboard content in redis
// that was sent from the client using post method to /api/v1/clipboard/{room_id}/store
// the content will be stored in redis lists with key {room_id}
pub async fn store_clipboard(
    payload: web::Json<ClipboardPayload>,
    path: web::Path<String>,
) -> impl Responder {
    let mut conn = get_redis_conn().await;
    let room_id = path.into_inner();
    let key = format!("{}:{}", REDIS_CLIPBOARD_KEY_PREFIX, room_id);

    let _: () = cmd("LPUSH")
        .arg(&key)
        .arg(payload.content.as_str())
        .query_async(&mut conn)
        .await
        .unwrap();

    // add the key to the list of keys
    // this is to make it easier to clean up the keys later
    // we will use set redis data type to store the keys
    let _: () = cmd("HSET")
        .arg(REDIS_CLIPBOARD_KEYS)
        .arg(payload.room_id.as_str()).arg(payload.room_name.as_str())
        .query_async(&mut conn)
        .await
        .unwrap();

    HttpResponse::Ok().finish()
}

// a function to get all room id in redis that was stored using redis set data type
// with key clipboard_keys
pub async fn get_clipboard_keys() -> impl Responder {
    let mut conn = get_redis_conn().await;
    let keys: HashMap<String, String> = cmd("HGETALL")
        .arg(REDIS_CLIPBOARD_KEYS)
        .query_async(&mut conn)
        .await
        .unwrap();

    // remove all REDIS_CLIPBOARD_KEY_PREFIX from the keys
    // and split the key to get the room_id and room_name
    let keys: Vec<RoomInfo> = keys.iter().map(|(k, v)| {
        let room_id = k.replace(REDIS_CLIPBOARD_KEY_PREFIX, "");
        RoomInfo {
            room_id: room_id.clone(),
            room_name: v.clone(),
        }
    }).collect();

    HttpResponse::Ok().json(keys)
}

// a function to get the clipboard content from redis
// that was sent from the client using get method to /api/v1/clipboard/{room_id}/get
// the content will be retrieved from redis lists with key {room_id}
pub async fn get_clipboard(path: web::Path<String>) -> impl Responder {
    let mut conn = get_redis_conn().await;
    let room_id = path.into_inner();
    let key = format!("{}:{}", REDIS_CLIPBOARD_KEY_PREFIX, room_id);

    // we use LRANGE to get the content of the list
    let content: Vec<String> = cmd("LRANGE")
        .arg(&key)
        .arg(0)
        .arg(-1)
        .query_async(&mut conn)
        .await
        .unwrap();

    HttpResponse::Ok().json(content)
}

extern crate log;

mod controller;
mod model;

use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use scylla::{Session, SessionBuilder};
use snowflake::SnowflakeIdBucket;
use std::env;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("𝖁𝖔𝖑𝖐𝖘𝖋𝖔𝖗𝖔");

    // Load .env values
    dotenv::dotenv().ok();

    println!("Building snowflakes.");
    // Snowflake ID generator
    // https://en.wikipedia.org/wiki/Snowflake_ID
    // https://crates.io/crates/rs-snowflake
    let snowflake = Data::new(Mutex::new(SnowflakeIdBucket::new(
        env::var("VF_NODE_ID")
            .expect("VF_NODE_ID is unset")
            .parse::<i32>()
            .expect("VF_NODE_ID is not i32"),
        env::var("VF_MACHINE_ID")
            .expect("VF_MACHINE_ID is unset")
            .parse::<i32>()
            .expect("VF_MACHINE_ID is not i32"),
    )));

    println!("Connecting to Scylla.");
    // Build Scylla connection
    let scylla = Data::new(
        SessionBuilder::new()
            .known_node(env::var("VF_DB_URI").expect("VF_DB_URI is unset"))
            //.user("username", "password")
            .build()
            .await
            .expect("Unable to connect to ScyllaDB"),
    );

    // Start webserver
    HttpServer::new(move || {
        App::new()
            .app_data(snowflake.clone())
            .app_data(scylla.clone())
            .service(get_index)
            .service(get_forum)
            .service(get_thread)
    })
    .bind(env::var("VF_APP_BIND").expect("VF_APP_BIND is unset"))?
    .run()
    .await
}

#[get("/")]
async fn get_index(scylla: Data<Session>) -> actix_web::Result<impl Responder> {
    let nodes = self::model::Node::fetch_all(scylla).await.unwrap();
    let mut strings = Vec::with_capacity(nodes.len());
    for node in nodes {
        strings.push(node.to_string());
    }

    Ok(HttpResponse::Ok().body(strings.join("<br />")))
}

#[get("/forums/{node_id}")]
async fn get_forum(
    scylla: Data<Session>,
    path: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    let node_id = path.into_inner();
    let nodes = self::model::Thread::fetch_node(scylla, node_id)
        .await
        .unwrap();
    let mut strings = Vec::with_capacity(nodes.len());
    for node in nodes {
        strings.push(node.to_string());
    }

    Ok(HttpResponse::Ok().body(strings.join("<br />")))
}

#[get("/threads/{thread_id}")]
async fn get_thread(
    scylla: Data<Session>,
    path: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    let thread_id = path.into_inner();
    let posts = self::model::Post::fetch_thread(scylla.clone(), thread_id, 1)
        .await
        .unwrap();
    let ugc = self::model::Ugc::fetch_many_posts(scylla.clone(), &posts)
        .await
        .unwrap();

    let mut strings = Vec::with_capacity(ugc.len());
    for post in &posts {
        if let Some(pugc) = ugc.get(&post.ugc_id) {
            strings.push(pugc.to_string());
        }
    }

    Ok(HttpResponse::Ok().body(strings.join("<hr />")))
}

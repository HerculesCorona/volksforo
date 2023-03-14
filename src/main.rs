//
// ^77        !~         .?7         :?!          :7!
// ~@@?      7@^         ~@#.        !@#        .5J?^
//  7@&:    :&7   :^:.   ^@#.   ..   !@B    ..  P@!     :^:     .  ..    :^:.
//   5@G    G5  !JYPB&5: ~@#. ~??GB7 !@B  ^5Y: !&@5! .7J5P##J  7&P?#B^ !JYP##5.
//   .#@?  JB  P#.  .5@G ~@# ^@G^^7^ !@B:Y5~    B@! :#5   ^#@? J@B.~^.PB.  .P@P
//    ~@@^^&^ ~@P    ^@P ~@#  7G##P! 7@#G@5.    B@! Y@7    ?@7 J@5   !@5    ^@5
//     Y@B#7  ^&@J:..?#^ !@# .7?:!@B 7@B.?&@J:  #@7 ?@&7:..PP  Y@P   ^@@J:..J#:
//     .GBY    ^5##GY!.  ~BP .Y#P??: !B5  .Y#Y. 5B~  !G#BP?~   ?BJ    ^5#BGJ!.
//

extern crate log;

mod controller;
mod middleware;
mod model;
mod view;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use middleware::context::Context;
use rand::{distributions::Alphanumeric, Rng};
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

    let secret_key = match std::env::var("VF_SESSION_KEY") {
        Ok(key) => Key::from(key.as_bytes()),
        Err(err) => {
            let random_string: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(128)
                .map(char::from)
                .collect();
            println!("VF_SESSION_KEY was invalid. Reason: {:?}\r\nThis means the key used for signing session cookies will invalidate every time the application is restarted. A secret key must be at least 64 bytes to be accepted.\r\n\r\nNeed a key? How about:\r\n{}", err, random_string);
            Key::from(random_string.as_bytes())
        }
    };

    // Start webserver
    HttpServer::new(move || {
        App::new()
            .app_data(snowflake.clone())
            .app_data(scylla.clone())
            .service(get_index)
            .service(get_forum)
            .service(get_thread)
            .wrap(Context::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::BAD_REQUEST, controller::error::render_400)
                    .handler(StatusCode::FORBIDDEN, controller::error::render_403)
                    .handler(StatusCode::NOT_FOUND, controller::error::render_404)
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        controller::error::render_500,
                    ),
            )
            .wrap(Logger::new("%a %{User-Agent}i"))
            .configure(controller::configure)
    })
    .bind(env::var("VF_APP_BIND").expect("VF_APP_BIND is unset"))?
    .run()
    .await
}

#[get("/")]
async fn get_index(context: Context, scylla: Data<Session>) -> actix_web::Result<impl Responder> {
    use self::model::Node;
    let nodes = Node::fetch_all(scylla).await.unwrap();
    Ok(self::view::IndexTemplate { context, nodes })
}

#[get("/forums/{node_id}/")]
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

#[get("/threads/{thread_id}/")]
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

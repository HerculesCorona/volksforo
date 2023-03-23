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
mod error;
mod filesystem;
mod filters;
mod middleware;
mod model;
mod session;
mod util;

#[cfg(test)]
mod test;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use env_logger::Env;
pub use error::Error;
use middleware::context::Context;
use rand::{distributions::Alphanumeric, Rng};
use scylla::SessionBuilder;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environmental variables and configure logging.
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    println!("𝖁𝖔𝖑𝖐𝖘𝖋𝖔𝖗𝖔");

    // Build Scylla connection
    log::info!("Connecting to Scylla.");
    let scylla = Data::new(
        SessionBuilder::new()
            .known_node(env::var("VF_DB_URI").expect("VF_DB_URI is unset"))
            //.user("username", "password")
            .build()
            .await
            .expect("Unable to connect to ScyllaDB"),
    );

    // Snowflake ID generator
    // The two env accepted must be unique in a federated cluster.
    // https://en.wikipedia.org/wiki/Snowflake_ID
    // https://crates.io/crates/rs-snowflake
    log::info!("Building snowflakes.");
    let _ = util::SNOWFLAKE_BUCKET.set(
        hexafreeze::Generator::new(
            env::var("VF_NODE_ID")
                .expect("VF_NODE_ID is unset")
                .parse::<i64>()
                .expect("VF_NODE_ID is not i32"),
            *hexafreeze::DEFAULT_EPOCH,
        )
        .expect("SNOWFLAKE_BUCKET failed to generate hexafreeze."),
    );

    log::info!("Building Argon2 hash config.");
    util::ARGON2_CONFIG
        .set(argon2::Config {
            variant: argon2::Variant::Argon2i,
            version: argon2::Version::Version13,
            mem_cost: 4096,
            time_cost: 3,
            lanes: 1,
            thread_mode: argon2::ThreadMode::Sequential,
            secret: &[],
            ad: &[],
            hash_length: 32,
        })
        .expect("ARGON2_CONFIG could not be set");

    log::info!("Validating session key.");
    let secret_key = match std::env::var("VF_SESSION_KEY") {
        Ok(key) => Key::from(key.as_bytes()),
        Err(err) => {
            let random_string: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(128)
                .map(char::from)
                .collect();
            log::warn!("VF_SESSION_KEY was invalid. Reason: {:?}\r\nThis means the key used for signing session cookies will invalidate every time the application is restarted. A secret key must be at least 64 bytes to be accepted.\r\n\r\nNeed a key? How about:\r\n{}", err, random_string);
            Key::from(random_string.as_bytes())
        }
    };

    // Start webserver
    HttpServer::new(move || {
        App::new()
            .app_data(scylla.clone())
            .wrap(Context::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(
                ErrorHandlers::new()
                    .default_handler(controller::error::error_document)
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        controller::error::render_500,
                    ),
            )
            .wrap(Logger::new("%a %{User-Agent}i"))
            // https://www.restapitutorial.com/lessons/httpmethods.html
            // GET    view_ (read/view/render entity)
            // GET    edit_ (get edit form)
            // PUT    create_
            // PATCH  update_ (apply edit)
            // Note: PUT and PATCH were added, removed, and re-added(?) to the HTML5 spec for <form method="">
            .configure(controller::configure)
    })
    .bind(env::var("VF_APP_BIND").expect("VF_APP_BIND is unset"))?
    .run()
    .await
}

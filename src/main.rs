extern crate actix_web;

extern crate serde;
extern crate serde_json;

extern crate env_logger;

#[macro_use]
extern crate serde_derive;

use actix_web::{server, App, Json, Result, http::Method};
use actix_web::middleware::Logger;

use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Words {
    words: Vec<String>
}

#[derive(Serialize, Debug)]
struct Counts {
    counts: HashMap<String, u32>
}

fn count_words(words: Words) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();

    for word in words.words {
        let counter = counts.entry(word).or_insert(0);
        *counter += 1;
    }

    counts
}

fn handle_words(words: Json<Words>) -> Result<Json<Counts>> {
    Ok(Json(Counts { counts: count_words(words.into_inner()) }))
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    server::new(
        || App::new()
            .middleware(Logger::default())
            .resource(
                "/words", |r| {
                    r.method(Method::POST)
                        .with(handle_words)
                        .limit(2097152);
                }
            )
    )
        .bind("127.0.0.1:8000")
        .expect("could not bind to 127.0.0.1:8000")
        .run();
}
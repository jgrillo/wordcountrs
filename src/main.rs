
extern crate actix_web;

extern crate serde;
extern crate serde_json;

extern crate env_logger;

extern crate futures;

extern crate bytes;

#[macro_use]
extern crate serde_derive;

use actix_web::{server, App, http::Method, Error, HttpRequest, HttpResponse, HttpMessage,
                AsyncResponder, Body};
use actix_web::middleware::Logger;

use env_logger::{Builder, Target};

use futures::Future;
use futures::stream::once;

use bytes::Bytes;

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Words<'a> {
    #[serde(borrow)]
    words: Vec<Cow<'a, str>>
}

#[derive(Serialize, Debug)]
struct Counts<'a> {
    #[serde(borrow)]
    counts: HashMap<&'a str, u32>
}

fn count_words<'a>(body: &'a Bytes) -> Result<Bytes, Error> {
    let words: Words<'a> = serde_json::from_slice(body.as_ref())?;
    let counts = words.words
        .iter()
        .fold(HashMap::default(),
              |mut counts, word| {
                  *counts.entry(word.as_ref()).or_insert(0) += 1;
                  counts
              });

    Ok(Bytes::from(serde_json::to_vec(&Counts { counts })?))
}

fn handle_words(req: &HttpRequest) -> Box<Future<Item=HttpResponse, Error=Error>> {
    req.body()
        .limit(5120000)
        .from_err()
        .and_then(|body: Bytes| -> Result<Bytes, Error> {
            count_words(&body)
        })
        .and_then(|response: Bytes| -> Result<HttpResponse, Error> {
            Ok(HttpResponse::Ok()
                .chunked()
                .content_length(response.len() as u64)
                .content_type("application/json")
                .body(Body::Streaming(Box::new(once(Ok(response))))))
        })
        .responder()
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    let mut env_logger_builder = Builder::new();
    env_logger_builder.target(Target::Stdout);
    if std::env::var("RUST_LOG").is_ok() {
        env_logger_builder.parse(&std::env::var("RUST_LOG").unwrap());
    }
    env_logger_builder.init();

    server::new(
        || App::new()
            .middleware(Logger::default())
            .resource(
                "/words",
                |r| r.method(Method::POST).f(handle_words)
            )
    )
        .bind("127.0.0.1:8000")
        .expect("could not bind to 127.0.0.1:8000")
        .run();
}

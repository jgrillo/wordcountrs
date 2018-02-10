#![feature(plugin)]

#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use rocket::Data;
use rocket_contrib::Json;

use serde::de::{SeqAccess, Deserialize, Deserializer, Visitor};

use std::collections::HashMap;
use std::marker::PhantomData;
use std::fmt;
use std::iter::Iterator;

// serde

#[derive(Debug, PartialEq)]
struct SeqIterator<'a, S: 'a>
    where S: SeqAccess<'a>
{
    seq: &'a S
}

impl <'a, S> SeqIterator<'a, S>
    where S: SeqAccess<'a>,
{
    fn new(seq: &'a S) -> SeqIterator<'a, S> {
        SeqIterator {seq}
    }
}

impl <'a, S> Iterator for SeqIterator<'a, S>
    where S: SeqAccess<'a>
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.seq.next_element() {
            Ok(element) => element,
            Err(err) => None
        }
    }
}

struct SeqVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for SeqVisitor<T>
    where T: Deserialize<'de> + 'de
{
    type Value = &'de SeqIterator<'de, S, Item=&'de T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of values.")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where S: SeqAccess<'de>
    {
        let seq_iter= &SeqIterator::new(&seq);
        Ok(seq_iter)
    }
}

fn deserialize_iter<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: Deserialize<'de>,
          D: Deserializer<'de>
{
    let visitor = SeqVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}

// data model

#[derive(Deserialize)]
struct Words<'a> {
    #[serde(deserialize_with = "deserialize_iter")]
    words: &'a SeqIterator<'a>
}

impl <'a> Words<'a> {
    fn new(words: &'a Iterator<Item=&'a str>) -> Words<'a> {
        Words { words }
    }
}

impl <'a> IntoIterator for Words<'a> {
    type Item = &'a str;
    type IntoIter = Box<impl Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(*self.words)
    }
}

#[derive(Serialize, Debug, PartialEq)]
struct Counts<'a> {
    counts: HashMap<&'a str, u32>
}

impl <'a> Counts <'a> {
    fn new(counts: HashMap<&'a str, u32>) -> Counts {
        Counts {
            counts
        }
    }
}

// handlers

// TODO: handle_words(data: Data) -> Stream<Counts>
#[post("/words", format = "application/json", data = "<data>")]
fn handle_words<'a>(data: Data) -> Json<Counts<'a>> {
    let reader = data.open();
    let words = serde_json::from_reader(reader)
        .expect("Failed to deserialize Words.");

    let counts = count_words(words);

    Json(Counts::new(counts))
}

// helpers

fn count_words<'a, S, V>(words: Words<'a>) -> HashMap<&'a str, u32>
    where S: SeqAccess<'a>,
          V: Visitor<'a>
{
    let mut counts: HashMap<&'a str, u32> = HashMap::new();

    for word in words {
        let counter = counts.entry(word).or_insert(0);
        *counter += 1;
    }

    counts
}

// server

fn main() {
    rocket::ignite()
        .mount("/words", routes![handle_words])
        .launch();
}

// tests

#[cfg(test)]
mod tests {
    use super::{Counts, Words, count_words};

    use std::collections::HashMap;

    #[test]
    fn test_count_words() {
        let words = Words::new(&vec![
            "word",
            "word",
            "wat",
            "word"
        ].iter());

        let mut expected_counts_map = HashMap::new();
        expected_counts_map.insert("word", 3);
        expected_counts_map.insert("wat", 1);

        let expected_counts = Counts::new(expected_counts_map);
        let counts_map = count_words(words);
        let counts = Counts::new(counts_map);

        assert_eq!(expected_counts, counts)
    }
}

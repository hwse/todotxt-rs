extern crate regex;
#[macro_use]
extern crate lazy_static;

use regex::{Regex};
use std::str::FromStr;

lazy_static! {
    static ref ENTRY_REGEX: Regex = Regex::new(
        r"^\s*((x)\s+)?(\(([[A-Z]])\)\s+)?((\d{4}-\d{2}-\d{2})\s+)?((\d{4}-\d{2}-\d{2})\s+)?(.*)$"
    ).unwrap();
}

#[derive(Eq, PartialEq, Debug, Default)]
#[allow(dead_code)]
pub struct TodoEntry {
    done: bool,
    priority: Option<char>,
    completion_date: Option<String>,
    creation_date: Option<String>,
    description: String,
}

#[derive(Eq, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Tag {
    Project(String),
    Context(String),
    KeyValue(String, String),
}

fn completion_and_creation_date(first_match: Option<String>,
                                second_match: Option<String>) -> (Option<String>, Option<String>) {
    match first_match {
        Some(first_date) => {
            match second_match {
                // Completion Date - Creation Date
                Some(second_date) => (Some(first_date), Some(second_date)),
                // Creation Date
                None => (None, Some(first_date))
            }
        },
        None => (None, None)
    }
}

impl FromStr for TodoEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = ENTRY_REGEX.captures(s) {
            let done = captures.get(2).is_some();
            let priority = captures.get(4)
                .map(|m| m.as_str().chars().nth(0).unwrap());
            let (completion_date, creation_date) = completion_and_creation_date(
                captures.get(6).map(|m| m.as_str().to_string()),
                captures.get(8).map(|m| m.as_str().to_string())
            );
            let description = captures.get(9)
                .map(|c| c.as_str().to_string()).unwrap_or_default();
            Ok(TodoEntry {
                done,
                priority,
                completion_date,
                creation_date,
                description,
            })
        } else {
            Err("regex did not match".to_string())
        }
    }
}

#[test]
fn test_from_str() {
    let expected = Ok(TodoEntry {
        done: true,
        priority: Some('A'),
        completion_date: None,
        creation_date: Some("2019-05-01".to_string()),
        description: "Get some milk".to_string()
    });
    let input = "x (A) 2019-05-01 Get some milk";
    assert_eq!(TodoEntry::from_str(input), expected);
}

impl TodoEntry {
    #[allow(dead_code)]
    fn tags(self) -> Vec<Tag> {
        self.description.split_whitespace()
            .map(|word| {
                if word.starts_with('+') {
                    Some(Tag::Project(word[1..].to_string()))
                } else if word.starts_with('@') {
                    Some(Tag::Context(word[1..].to_string()))
                } else if let Some(index) = word.find(":") {
                    let (key, value) = word.split_at(index);
                    Some(Tag::KeyValue(key.to_string(), value[1..].to_string()))
                } else {
                    None
                }
            })
            .filter(|tag| tag.is_some())
            .map(|tag| tag.unwrap())
            .collect()
    }
}

#[test]
fn test_tags() {
    let entry = TodoEntry {
        done: false,
        priority: None,
        completion_date: None,
        creation_date: None,
        description: "Do Homework due:2019-02-01 @at_home +school".to_string()
    };
    let tags = entry.tags();
    let result = vec![Tag::KeyValue("due".to_string(), "2019-02-01".to_string()),
                      Tag::Context("at_home".to_string()), Tag::Project("school".to_string())];
    assert_eq!(tags, result)
}

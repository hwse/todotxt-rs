extern crate regex;
#[macro_use]
extern crate lazy_static;

use regex::{Regex};
use std::str::FromStr;
use std::fmt::{Display, Formatter, Error, Write};

lazy_static! {
    static ref ENTRY_REGEX: Regex = Regex::new(
        r"^\s*((x)\s+)?(\(([[A-Z]])\)\s+)?((\d{4}-\d{2}-\d{2})\s+)?((\d{4}-\d{2}-\d{2})\s+)?(.*)$"
    ).unwrap();
}

// group indices of ENTRY_REGEX
const DONE_INDEX: usize = 2;
const PRIORITY_INDEX: usize = 4;
const FIRST_DATE_INDEX: usize = 6;
const SECOND_DATE_INDEX: usize = 8;
const DESCRIPTION_INDEX: usize = 9;

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
            let done = captures.get(DONE_INDEX).is_some();
            let priority = captures.get(PRIORITY_INDEX)
                .map(|m| m.as_str().chars().nth(0).unwrap());
            let (completion_date, creation_date) = completion_and_creation_date(
                captures.get(FIRST_DATE_INDEX).map(|m| m.as_str().to_string()),
                captures.get(SECOND_DATE_INDEX).map(|m| m.as_str().to_string())
            );
            let description = captures.get(DESCRIPTION_INDEX)
                .map(|c| c.as_str().to_string()).unwrap_or_default();
            Ok(TodoEntry {
                done,
                priority,
                completion_date,
                creation_date,
                description,
            })
        } else {
            // this should never happen
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

impl Display for TodoEntry {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.done {
            f.write_str("x ")?;
        }
        if let Some(priority) = self.priority {
            f.write_fmt(format_args!("({}) ", priority))?;
        };
        if let Some(completion_date) = &self.completion_date {
            f.write_str(completion_date.as_str())?;
            f.write_char(' ')?;
        }
        if let Some(creation_date) = &self.creation_date {
            f.write_str(creation_date.as_str())?;
            f.write_char(' ')?;
        }
        f.write_str(self.description.as_str())?;
        Ok(())
    }
}

#[test]
fn test_to_string() {
    let entry = TodoEntry {
        done: true,
        priority: Some('A'),
        completion_date: None,
        creation_date: Some("2019-05-01".to_string()),
        description: "Get some milk".to_string()
    };
    assert_eq!("x (A) 2019-05-01 Get some milk", entry.to_string());
}

#[test]
fn test_parse_and_to_string() {
    // well formed entry should be same after parse and to_string
    let entry_as_text = "x (B) 2019-07-02 2019-06-03 Do Stuff @tag1 @tag2 +project k:v";
    let entry = TodoEntry::from_str(entry_as_text).unwrap();
    assert_eq!(entry_as_text, entry.to_string().as_str());
}

impl TodoEntry {
    #[allow(dead_code)]
    fn tags(self) -> Vec<Tag> {
        self.description
            .split_whitespace()
            .filter_map(|word| {
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

#[test]
fn test_parse_and_tags() {
    let entry = TodoEntry::from_str("x (B) 2019-07-02 2019-06-03 Do Stuff @tag1 @tag2 +project k:v").unwrap();

    assert_eq!(true, entry.done);
    assert_eq!(Some('B'), entry.priority);
    assert_eq!(Some("2019-07-02".to_string()), entry.completion_date);
    assert_eq!(Some("2019-06-03".to_string()), entry.creation_date);
    assert_eq!("Do Stuff @tag1 @tag2 +project k:v", entry.description);
    assert_eq!(vec![Tag::Context("tag1".to_string()),
                    Tag::Context("tag2".to_string()),
                    Tag::Project("project".to_string()),
                    Tag::KeyValue("k".to_string(), "v".to_string())],
               entry.tags());
}
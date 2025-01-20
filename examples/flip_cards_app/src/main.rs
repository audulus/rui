use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashSet, VecDeque};

fn find_flip_cards(value: &Value) -> Vec<Value> {
    let mut flip_cards = Vec::new();

    // Define sets of possible keywords for "question" and "answer"
    let question_keywords: HashSet<&str> = ["question", "q", "frontside"].iter().cloned().collect();
    let answer_keywords: HashSet<&str> = ["answer", "a", "backside", "explanation", "solution"]
        .iter()
        .cloned()
        .collect();

    // Use a stack (VecDeque) to mimic recursion
    let mut stack = VecDeque::new();
    stack.push_back(value);

    // While there are items on the stack, process them
    while let Some(current_value) = stack.pop_front() {
        match current_value {
            Value::Object(map) => {
                let mut question = None;
                let mut answer = None;

                for (key, val) in map {
                    // Check if the key matches any question keyword
                    if question_keywords.contains(&key.to_lowercase().as_str()) {
                        if let Some(q) = val.as_str() {
                            question = Some(q.to_string());
                        }
                    }

                    // Check if the key matches any answer keyword
                    if answer_keywords.contains(&key.to_lowercase().as_str()) {
                        if let Some(a) = val.as_str() {
                            answer = Some(a.to_string());
                        }
                    }

                    // Add the value to the stack if it is an object or array (to process nested structures)
                    if val.is_object() || val.is_array() {
                        stack.push_back(val);
                    }
                }

                // If both question and answer were found, add the flip card
                if let (Some(q), Some(a)) = (question, answer) {
                    flip_cards.push(Value::Object({
                        let mut map = Map::new();
                        map.insert("q".to_string(), Value::String(q));
                        map.insert("a".to_string(), Value::String(a));
                        map
                    }));
                }
            }
            Value::Array(arr) => {
                // If the current value is an array, push its elements to the stack
                for item in arr {
                    stack.push_back(item);
                }
            }
            _ => {}
        }
    }

    flip_cards
}

use rui::*;

#[derive(Serialize, Deserialize)]
struct FlipCard {
    question: String,
    answer: String,
}

fn flip_card_view(value: &Value) -> impl View {
    let value = value.clone();
    state(
        move || {
            (
                false,
                value["q"].as_str().unwrap().to_string(),
                value["a"].as_str().unwrap().to_string(),
            )
        },
        |s, cx| {
            vstack((
                text(if cx[s].0 {
                    cx[s].1.as_str()
                } else {
                    cx[s].2.as_str()
                })
                .font_size(12)
                .padding(Auto),
                button(
                    if cx[s].0 {
                        "Hide Answer"
                    } else {
                        "Show Answer"
                    },
                    move |cx| {
                        cx[s].0 = !cx[s].0;
                    },
                )
                .padding(Auto),
            ))
        },
    )
}

use rust_search::SearchBuilder;
use std::fs;

fn main() {
    let mut search: Vec<String> = SearchBuilder::default()
        .location(".")
        .search_input("flip_card_data")
        .ext("json")
        .depth(10)
        .build()
        .collect();

    let data = fs::read_to_string(search.pop().unwrap()).expect("Failed to read data file");

    // Deserialize the raw string into a serde_json::Value
    let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");

    // Find the flip cards by iteratively searching the JSON
    let flip_cards = find_flip_cards(&value);
    list(flip_cards, flip_card_view).run();
}

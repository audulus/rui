use rui::*;
use rust_search::SearchBuilder;
use serde_json::{Map, Value};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::sync::Arc;

// Function to find and extract flip cards from JSON data
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
                        let mut map = Map::with_capacity(2);
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

// Struct to represent the state of a flip card
#[derive(Clone)]
struct FlipCardState {
    show_answer: bool,
    question: Arc<str>,
    answer: Arc<str>,
}

// Struct to represent the state of all flip cards
struct FlipCardsState {
    flip_cards: Vec<FlipCardState>,
    current_index: usize,
}

// Main function
fn main() {
    // Search for flip card JSON data
    let mut search: Vec<String> = SearchBuilder::default()
        .location(".")
        .search_input("flip_card_data")
        .ext("json")
        .depth(10)
        .build()
        .collect();

    // Read the JSON file containing the flip cards data
    let data = fs::read_to_string(search.pop().unwrap()).expect("Failed to read data file");

    // Deserialize the raw string into a serde_json::Value
    let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");

    // Find the flip cards by iteratively searching the JSON
    let flip_cards = find_flip_cards(&value);

    // Initialize the state with flip cards and the first card
    vstack((
        text("Flip Cards").font_size(20).padding(Auto),
        spacer().size([0.0, 20.0]),
        state(
            move || FlipCardsState {
                flip_cards: flip_cards
                    .iter()
                    .map(|card| FlipCardState {
                        show_answer: false,
                        question: card["q"].to_string().into(),
                        answer: card["a"].to_string().into(),
                    })
                    .collect(),
                current_index: 0, // Start at the first card
            },
            |s, cx| {
                let flip_cards = &cx[s].flip_cards;
                let current_index = cx[s].current_index;
                let current_card = &flip_cards[current_index];

                // Render the current flip card
                vstack((
                    text(if current_card.show_answer {
                        &current_card.answer
                    } else {
                        &current_card.question
                    })
                    .font_size(12)
                    .padding(Auto),
                    button(
                        if current_card.show_answer {
                            "Hide Answer"
                        } else {
                            "Show Answer"
                        },
                        move |cx| {
                            cx[s].flip_cards[current_index].show_answer =
                                !cx[s].flip_cards[current_index].show_answer;
                        },
                    )
                    .padding(Auto),
                    button("Next Card", move |cx| {
                        cx[s].current_index = (cx[s].current_index + 1) % cx[s].flip_cards.len(); // Move to next card
                        cx[s].flip_cards[current_index].show_answer = false;
                        // Reset the answer visibility
                    })
                    .padding(Auto),
                ))
            },
        ),
    ))
    .run();
}

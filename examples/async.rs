use rui::*;
use std::{thread, time};

fn main() {
    rui(state(
        || "task not started".to_string(),
        |s| {
            let txt = s.get();
            hstack((
                button(text("press to begin"), move || {
                    let s2 = s.clone();
                    thread::spawn(move || {
                        s2.set("task started".to_string());
                        thread::sleep(time::Duration::from_secs(2));
                        s2.set("task complete".to_string());
                    });
                }),
                text(&txt),
            ))
        },
    ));
}

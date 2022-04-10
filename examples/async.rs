use rui::*;
use std::{thread, time};

fn main() {
    rui(state(
        || "task not started".to_string(),
        |s| {
            let txt = s.get();
            hstack((
                button(text("press to begin"), move || {
                    thread::spawn(move || {
                        on_main(move || s.set("task started".into()) );
                        thread::sleep(time::Duration::from_secs(2));
                        on_main(move || s.set("task complete".into()) );
                    });
                }),
                text(&txt),
            ))
        },
    ));
}

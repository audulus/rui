use rui::*;
use std::{thread, time};

fn main() {
    rui(
        state("task not started".to_string(), |s| {
            let txt = s.get();
            hstack((
                button(text("press to begin"), move || {
                    s.set("task started".to_string());
                    let s2 = s.clone();

                    thread::spawn(move || {
                        thread::sleep(time::Duration::from_secs(2));
                        s2.with_mut(|s| *s = "task complete".to_string());
                    });
                }),
                text(&txt)
            ))
        })
    );
}

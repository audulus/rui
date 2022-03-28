use rui::*;
use std::{thread, time};

fn main() {
    rui(
        state("task not started".to_string(), |s| {
            let txt = s.get();
            hstack((
                button(text("press to begin"), move || {
                    println!("beginning task");
                    s.with_mut(|s| *s = "task started".to_string());
                    let s2 = s.clone();

                    thread::spawn(move || {
                        // thread code
                        println!("inside task");

                        thread::sleep(time::Duration::from_secs(2));

                        println!("task finished");

                        s2.with_mut(|s| *s = "task complete".to_string());
                    });
                }),
                text(&txt)
            ))
        })
    );
}

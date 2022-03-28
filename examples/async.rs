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

                    thread::spawn(|| {
                        // thread code
                        println!("inside task");

                        thread::sleep(time::Duration::from_secs(2));

                        println!("task finished");
                    });
                }),
                text(&txt)
            ))
        })
    );
}

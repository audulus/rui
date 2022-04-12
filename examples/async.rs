use rui::*;
use std::{
    thread::{sleep, spawn},
    time::Duration,
};

fn main() {
    rui(state(
        || "task not started".to_string(),
        |s, cx| {
            hstack((
                button(text("press to begin"), move |_| {
                    spawn(move || {
                        on_main(move |cx| cx[s] = "task started".into());
                        sleep(Duration::from_secs(2));
                        on_main(move |cx| cx[s] = "task complete".into());
                    });
                }),
                text(&cx[s]),
            ))
        },
    ));
}

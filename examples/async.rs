use rui::*;
use std::{
    thread::{sleep, spawn},
    time::Duration,
};

fn main() {
    rui(state(
        || "task not started".to_string(),
        |s, cx| {
            let txt = &cx[s];
            hstack((
                button(text("press to begin"), move |_cx| {
                    spawn(move || {
                        on_main(move |cx| cx[s] = "task started".into());
                        sleep(Duration::from_secs(2));
                        on_main(move |cx| cx[s] = "task complete".into());
                    });
                }),
                text(&txt),
            ))
        },
    ));
}

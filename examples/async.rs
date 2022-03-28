use rui::*;

fn main() {
    rui(
        state("task not started".to_string(), |s| {
            let txt = s.get();
            hstack((
                button(text("press to begin"), move || {
                    println!("beginning task");
                    s.with_mut(|s| *s = "task started".to_string());
                }),
                text(&txt)
            ))
        })
    );
}

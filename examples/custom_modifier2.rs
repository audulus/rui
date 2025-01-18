use rui::*;

trait MyMod: View + Sized {
    fn my_modifier(self) -> impl View {
        self.offset(LocalOffset::new(0.0, 100.0))
    }
}

impl<V: View> MyMod for V {}

fn my_text(name: &str) -> impl View {
    let name = name.to_string();
    text(name.as_str())
}

fn main() {
    hstack((my_text("without"), my_text("with").my_modifier())).run()
}
use rui::*;

trait MyMod {
    fn my_modifier(self) -> AnyView;
}

impl MyMod for AnyView {
    fn my_modifier(self) -> AnyView {
        any_view(self.offset(LocalOffset::new(0.0, 100.0)))
    }
}

fn my_text(name: &str) -> AnyView {
    let name = name.to_string();
    any_view(text(name.as_str()))
}

fn main() {
    hstack((my_text("without"), my_text("with").my_modifier())).run();
}

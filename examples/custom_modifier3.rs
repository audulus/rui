
trait MyMod: View + Sized {
    fn my_modifier(self) -> impl View {
        self.offset(LocalOffset::new(0.0, 100.0))
    }
}

impl<V: View> MyMod for V {}

fn main() {
    hstack((text("without"), text("with").my_modifier())).run()
}

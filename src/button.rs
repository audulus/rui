pub use crate::*;

pub struct Button {
    text: String,
    func: Box<dyn Fn()>,
}

impl View for Button {
    fn draw(&self, _id: ViewID, _cx: &mut Context) {
        println!("Button({:?})", self.text);
    }
    fn process(&self, event: &Event, _id: ViewID, _cx: &mut Context) {
        match event {
            Event::PressButton(name) => {
                if *name == self.text {
                    (*self.func)();
                }
            }
        }
    }
}

pub fn button<F: Fn() + 'static>(name: &str, f: F) -> Button {
    Button {
        text: String::from(name),
        func: Box::new(f),
    }
}

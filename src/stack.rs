
use crate::*;

pub struct Stack {
    children: Vec<Box<dyn View>>,
}

impl View for Stack {

    fn draw(&self, id: ViewID) {
        println!("Stack {{");
        let mut c: u16 = 0;
        for child in &self.children {
            (*child).draw(id.child(c));
            c += 1;
        }
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID) {
        let mut c: u16 = 0;
        for child in &self.children {
            (*child).process(event, id.child(c));
            c += 1;
        }
    }

}

impl Stack {
    pub fn new() -> Self {
        Self { children: vec![] }
    }

    pub fn push(&mut self, view: impl View + 'static) {
        self.children.push(Box::new(view))
    }
}

pub struct Stack2<V0: View, V1: View> {
    children: (V0, V1)
}

impl<V0, V1> View for Stack2<V0, V1> where V0:View, V1:View {

    fn draw(&self, id: ViewID) {
        println!("Stack {{");
        self.children.0.draw(id.child(0));
        self.children.1.draw(id.child(1));
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID) {
        self.children.0.process(event, id.child(0));
        self.children.1.process(event, id.child(1));
    }

}

pub fn stack2(v0: impl View + 'static, v1: impl View + 'static) -> impl View {
    Stack2{ children: (v0, v1) }
}

pub struct Stack3<V0: View, V1: View, V2: View> {
    children: (V0, V1, V2)
}

impl<V0, V1, V2> View for Stack3<V0, V1, V2> where V0:View, V1:View, V2:View {

    fn draw(&self, id: ViewID) {
        println!("Stack {{");
        self.children.0.draw(id.child(0));
        self.children.1.draw(id.child(1));
        self.children.2.draw(id.child(2));
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID) {
        self.children.0.process(event, id.child(0));
        self.children.1.process(event, id.child(1));
        self.children.2.process(event, id.child(2));
    }

}

pub fn stack3(v0: impl View + 'static, v1: impl View + 'static, v2: impl View + 'static) -> impl View {
    Stack3{ children: (v0, v1, v2) }
}
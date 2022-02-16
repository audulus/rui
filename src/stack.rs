use crate::*;

pub struct Stack {
    children: Vec<Box<dyn View>>,
}

impl View for Stack {
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Stack {{");
        let mut c: u16 = 0;
        for child in &self.children {
            (*child).print(id.child(c), cx);
            c += 1;
        }
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context) {
        let mut c: u16 = 0;
        for child in &self.children {
            (*child).process(event, id.child(c), cx);
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

#[macro_export]
macro_rules! stack {
    ( $( $x:expr );* ) => {
        {
            let mut temp_stack = Stack::new();
            $(
                temp_stack.push($x);
            )*
            temp_stack
        }
    };
}

pub struct Stack2<V0: View, V1: View> {
    children: (V0, V1),
}

impl<V0, V1> View for Stack2<V0, V1>
where
    V0: View,
    V1: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Stack {{");
        self.children.0.print(id.child(0), cx);
        self.children.1.print(id.child(1), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context) {
        self.children.0.process(event, id.child(0), cx);
        self.children.1.process(event, id.child(1), cx);
    }
}

pub fn stack2(v0: impl View + 'static, v1: impl View + 'static) -> impl View {
    Stack2 { children: (v0, v1) }
}

pub struct Stack3<V0: View, V1: View, V2: View> {
    children: (V0, V1, V2),
}

impl<V0, V1, V2> View for Stack3<V0, V1, V2>
where
    V0: View,
    V1: View,
    V2: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Stack {{");
        self.children.0.print(id.child(0), cx);
        self.children.1.print(id.child(1), cx);
        self.children.2.print(id.child(2), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context) {
        self.children.0.process(event, id.child(0), cx);
        self.children.1.process(event, id.child(1), cx);
        self.children.2.process(event, id.child(2), cx);
    }
}

pub fn stack3(
    v0: impl View + 'static,
    v1: impl View + 'static,
    v2: impl View + 'static,
) -> impl View {
    Stack3 {
        children: (v0, v1, v2),
    }
}

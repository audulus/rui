use crate::*;

pub enum StackOrientation {
    Horizontal,
    Vertical,
}

pub struct Stack {
    orientation: StackOrientation,
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

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c: u16 = 0;
        for child in &self.children {
            (*child).draw(id.child(c), cx, vger);
            c += 1;
        }
    }
}

impl Stack {
    pub fn new(orientation: StackOrientation) -> Self {
        Self { orientation, children: vec![] }
    }

    pub fn push(&mut self, view: impl View + 'static) {
        self.children.push(Box::new(view))
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context) -> LocalSize {
        let n = self.children.len() as f32;
        let proposed_child_size = match self.orientation {
            StackOrientation::Horizontal => LocalSize::new(sz.width / n, sz.height),
            StackOrientation::Vertical => LocalSize::new(sz.width, sz.height/n),
        };

        let child_sizes: Vec<LocalSize> = vec![];
        for child in &self.children {
            // layout each child
        }
        
        // Calculate child offsets.

        // Return final size.
        let width_sum:f32 = child_sizes.iter().map(|&sz| sz.width).sum();
        let height_sum:f32 = child_sizes.iter().map(|&sz| sz.height).sum();
        
        match self.orientation {
            StackOrientation::Horizontal => LocalSize::new(width_sum, sz.height),
            StackOrientation::Vertical => LocalSize::new(sz.width, height_sum),
        }
    }
}

#[macro_export]
macro_rules! hstack {
    ( $( $x:expr );* ) => {
        {
            let mut temp_stack = Stack::new(StackOrientation::Horizontal);
            $(
                temp_stack.push($x);
            )*
            temp_stack
        }
    };
}

#[macro_export]
macro_rules! vstack {
    ( $( $x:expr );* ) => {
        {
            let mut temp_stack = Stack::new(StackOrientation::Vertical);
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

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.children.0.draw(id.child(0), cx, vger);
        self.children.1.draw(id.child(1), cx, vger);
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

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.children.0.draw(id.child(0), cx, vger);
        self.children.1.draw(id.child(1), cx, vger);
        self.children.2.draw(id.child(2), cx, vger);
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

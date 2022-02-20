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
            let child_id = id.child(c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            let mut local_event = event.clone();
            local_event.position -= offset;

            (*child).process(&local_event, child_id, cx);
            c += 1;
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c: u16 = 0;
        for child in &self.children {
            let child_id = id.child(c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            vger.save();

            vger.translate(offset);

            (*child).draw(child_id, cx, vger);
            c += 1;

            vger.restore();
        }
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let n = self.children.len() as f32;

        match self.orientation {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(sz.width / n, sz.height);

                let mut c: u16 = 0;
                let mut width_sum = 0.0;
                for child in &self.children {
                    let child_id = id.child(c);
                    let child_size = child.layout(child_id, proposed_child_size, cx, vger);

                    cx.layout.entry(child_id).or_default().offset =
                        [width_sum, (sz.height - child_size.height) / 2.0].into();

                    width_sum += child_size.width;
                    c += 1;
                }

                LocalSize::new(width_sum, sz.height)
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(sz.width, sz.height / n);

                let mut c: u16 = 0;
                let mut y = sz.height;
                let mut height_sum = 0.0;
                for child in &self.children {
                    let child_id = id.child(c);
                    let child_size = child.layout(child_id, proposed_child_size, cx, vger);

                    y -= child_size.height;
                    cx.layout.entry(child_id).or_default().offset =
                        [(sz.width - child_size.width) / 2.0, y].into();

                    height_sum += child_size.height;
                    c += 1;
                }

                LocalSize::new(sz.width, height_sum)
            }
        }
    }
}

impl Stack {
    pub fn new(orientation: StackOrientation) -> Self {
        Self {
            orientation,
            children: vec![],
        }
    }

    pub fn push(&mut self, view: impl View + 'static) {
        self.children.push(Box::new(view))
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

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        // TODO
        [0.0, 0.0].into()
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

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        // TODO
        [0.0, 0.0].into()
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

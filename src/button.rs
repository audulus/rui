pub use crate::*;

pub struct Button {
    text: String,
    func: Box<dyn Fn()>,
}

impl Button {
    pub const DEFAULT_SIZE: u32 = 18;
}

impl View for Button {
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("Button({:?})", self.text);
    }
    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context) {
        match &event.kind {
            EventKind::PressButton(name) => {
                if *name == self.text {
                    (*self.func)();
                }
            },
            EventKind::TouchBegin{id} => {
                if cx.layout.entry(vid).or_default().rect.contains(event.position) {
                    (*self.func)();
                }
            },
            _ => (),
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {

        // XXX: obviously need to use vger to compute text size
        let size = LocalSize::new(self.text.len() as f32 * 10.0, 10.0);

        let paint = vger.color_paint(Color{r: 0.1, g: 0.1, b: 0.1, a: 1.0});
        vger.fill_rect(LocalPoint::zero(), [size.width, size.height].into(), 4.0, paint);

        vger.text(self.text.as_str(), Button::DEFAULT_SIZE, Color::MAGENTA, None);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        // XXX: obviously need to use vger to compute text size
        let size = LocalSize::new(self.text.len() as f32 * 10.0, 10.0);

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), size),
                offset: LocalOffset::zero(),
            },
        );
        size
    }
}

pub fn button<F: Fn() + 'static>(name: &str, f: F) -> Button {
    Button {
        text: String::from(name),
        func: Box::new(f),
    }
}

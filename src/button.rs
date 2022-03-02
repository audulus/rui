pub use crate::*;

pub struct Button<F> {
    text: String,
    func: F,
}

impl<F> Button<F> {
    pub const DEFAULT_SIZE: u32 = 18;
    pub const PADDING: f32 = 4.0;
}

impl<F> View for Button<F>
where
    F: Fn(),
{
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("Button({:?})", self.text);
    }
    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context, _vger: &mut VGER) {
        match &event.kind {
            EventKind::PressButton(name) => {
                if *name == self.text {
                    (self.func)();
                }
            }
            EventKind::TouchBegin { id } => {
                if cx
                    .layout
                    .entry(vid)
                    .or_default()
                    .rect
                    .contains(event.position)
                {
                    (self.func)();
                }
            }
            _ => (),
        }
    }

    fn draw(&self, _id: ViewID, _cx: &mut Context, vger: &mut VGER) {
        let bounds = vger.text_bounds(self.text.as_str(), Button::<F>::DEFAULT_SIZE, None);
        let padding = LocalSize::new(Button::<F>::PADDING, Button::<F>::PADDING);

        let paint = vger.color_paint(Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        });
        vger.fill_rect(
            LocalPoint::zero(),
            LocalPoint::zero() + bounds.size + padding * 2.0,
            4.0,
            paint,
        );

        vger.save();
        vger.translate(
            -LocalOffset::new(bounds.origin.x, bounds.origin.y)
                + LocalOffset::new(Button::<F>::PADDING, Button::<F>::PADDING),
        );

        vger.text(
            self.text.as_str(),
            Button::<F>::DEFAULT_SIZE,
            TEXT_COLOR,
            None,
        );

        vger.restore();
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let padding = LocalSize::new(Button::<F>::PADDING, Button::<F>::PADDING);
        let size = vger
            .text_bounds(self.text.as_str(), Button::<F>::DEFAULT_SIZE, None)
            .size
            + padding * 2.0;

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), size),
                offset: LocalOffset::zero(),
            },
        );
        size
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewID> {
        if cx.layout.entry(id).or_default().rect.contains(pt) {
            Some(id)
        } else {
            None
        }
    }
}

/// Create a button with a label and an action.
pub fn button<F: Fn() + 'static>(name: &str, f: F) -> Button<F> {
    Button {
        text: String::from(name),
        func: f,
    }
}

pub const BUTTON_CORNER_RADIUS: f32 = 5.0;

pub fn button2<F: Fn() + 'static>(name: &str, f: F) -> impl View {
    text(&name)
        .padding(Auto)
        .background(rectangle().corner_radius(BUTTON_CORNER_RADIUS).color(BUTTON_BACKGROUND_COLOR))
        .tap(f)
}

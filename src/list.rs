use crate::*;
use std::hash::Hash;

pub struct List<ID: Hash, V:View, F: Fn(&ID) -> V> {
    ids: Vec<ID>,
    func: F,
}

impl<ID, V, F> View for List<ID, V, F> 
where
    ID: Hash,
    V: View,
    F: Fn(&ID) -> V {

    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("List {{");
        let mut c = 0;
        for child in &self.ids {
            ((self.func)(child)).print(id.child(&c), cx);
            c += 1;
        }
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c: u16 = 0;
        for child in &self.ids {
            let child_id = id.child(&c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            let mut local_event = event.clone();
            local_event.position -= offset;

            ((self.func)(child)).process(&local_event, child_id, cx, vger);
            c += 1;
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c: u16 = 0;
        for child in &self.ids {
            let child_id = id.child(&c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            vger.save();

            vger.translate(offset);

            ((self.func)(child)).draw(child_id, cx, vger);
            c += 1;

            vger.restore();
        }
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let n = self.ids.len() as f32;
        let proposed_child_size = LocalSize::new(sz.width, sz.height / n);

        let mut c: u16 = 0;
        let mut y = sz.height;
        let mut height_sum = 0.0;
        for child in &self.ids {
            let child_id = id.child(&c);
            let child_size = ((self.func)(child)).layout(child_id, proposed_child_size, cx, vger);

            y -= child_size.height;
            cx.layout.entry(child_id).or_default().offset =
                [(sz.width - child_size.width) / 2.0, y].into();

            height_sum += child_size.height;
            c += 1;
        }

        LocalSize::new(sz.width, height_sum)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        let mut c: u16 = 0;
        let mut hit = None;
        for child in &self.ids {
            let child_id = id.child(&c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            if let Some(h) = ((self.func)(child)).hittest(child_id, pt - offset, cx, vger) {
                hit = Some(h)
            }

            c += 1;
        }
        hit
    }
}
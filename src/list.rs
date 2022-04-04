use crate::*;
use std::hash::Hash;

pub struct List<ID: Hash, V: View, F: Fn(&ID) -> V> {
    ids: Vec<ID>,
    func: F,
}

impl<ID, V, F> View for List<ID, V, F>
where
    ID: Hash,
    V: View,
    F: Fn(&ID) -> V,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("List {{");
        for child in &self.ids {
            ((self.func)(child)).print(id.child(child), cx);
        }
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            let mut local_event = event.clone();
            local_event.position -= offset;

            ((self.func)(child)).process(&local_event, child_id, cx, vger);
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            vger.save();

            vger.translate(offset);

            ((self.func)(child)).draw(child_id, cx, vger);

            vger.restore();
        }
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let n = self.ids.len() as f32;
        let proposed_child_size = LocalSize::new(sz.width, sz.height / n);

        let mut y = sz.height;
        let mut height_sum = 0.0;
        for child in &self.ids {
            let child_id = id.child(child);
            let child_size = ((self.func)(child)).layout(child_id, proposed_child_size, cx, vger);

            y -= child_size.height;
            cx.layout.entry(child_id).or_default().offset =
                [(sz.width - child_size.width) / 2.0, y].into();

            height_sum += child_size.height;
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
        let mut hit = None;
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            if let Some(h) = ((self.func)(child)).hittest(child_id, pt - offset, cx, vger) {
                hit = Some(h)
            }
        }
        hit
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        for child in &self.ids {
            let child_id = id.child(child);
            ((self.func)(child)).commands(child_id, cx, cmds)
        }
    }

    fn mark(&self, id: ViewID, cx: &mut Context) {
        for child in &self.ids {
            ((self.func)(child)).mark(id.child(child), cx)
        }
    }
}

/// Displays a list of items all of which are represented by the same View. See `examples/list.rs`.
///
/// `ids` is a Vec of items that implement Hash.
///
/// `f` is a function called to generate a View for each item.
///
/// For example:
///
/// ```no_run
/// # use rui::*;
/// rui(list(vec![1, 2, 3], |i| {
///     hstack((
///         circle(),
///         text(&format!("{:?}", i))
///     ))
/// }));
/// ```
pub fn list<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, V, F> {
    List { ids, func: f }
}

use crate::*;
use std::hash::Hash;

pub struct List<ID, F> {
    ids: Vec<ID>,
    func: F,
}

impl<ID, V, F> View for List<ID, F>
where
    ID: Hash + 'static,
    V: View,
    F: Fn(&ID) -> V + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("List {{");
        for child in &self.ids {
            ((self.func)(child)).print(id.child(child), cx);
        }
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;

            let mut local_event = event.clone();
            local_event.position -= offset;

            ((self.func)(child)).process(&local_event, child_id, cx, vger);
        }
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;

            vger.save();

            vger.translate(offset);

            ((self.func)(child)).draw(child_id, cx, vger);

            vger.restore();
        }
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
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

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context, region: &mut Region<WorldSpace>) {
        
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;
            let xf = xform.pre_translate(offset.into());
            ((self.func)(child)).dirty(id, xf, cx, region);
        }
        
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        let mut hit = None;
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;

            if let Some(h) = ((self.func)(child)).hittest(child_id, pt - offset, cx, vger) {
                hit = Some(h)
            }
        }
        hit
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        for child in &self.ids {
            let child_id = id.child(child);
            ((self.func)(child)).commands(child_id, cx, cmds)
        }
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        for child in &self.ids {
            ((self.func)(child)).gc(id.child(child), cx, map)
        }
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let mut node = accesskit::Node::new(id.access_id(), accesskit::Role::List);
        for child in &self.ids {
            if let Some(i) = ((self.func)(child)).access(id.child(child), cx, nodes) {
                node.children.push(i)
            }
        }
        nodes.push(node);
        Some(id.access_id())
    }
}

impl<ID, F> private::Sealed for List<ID, F> {}

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
pub fn list<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List { ids, func: f }
}

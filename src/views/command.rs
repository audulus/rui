use crate::*;
use std::{any::Any, sync::Arc};

#[derive(Clone)]
pub struct Command<V, F> {
    child: V,
    name: Arc<str>,
    key: Option<HotKey>,
    func: F,
}

impl<V, F> Command<V, F>
where
    V: View,
    F: Fn(&mut Context) + Clone + 'static,
{
    pub fn new(v: V, name: Arc<str>, key: Option<HotKey>, f: F) -> Self {
        Self {
            child: v,
            name,
            key,
            func: f,
        }
    }
}

impl<V, F> DynView for Command<V, F>
where
    V: View,
    F: Fn(&mut Context) + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Command(name) = &event
            && *name == self.name
        {
            (self.func)(cx);
        }
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
        cmds.push(CommandInfo {
            path: self.name.clone(),
            key: self.key,
        })
    }

    fn access(
        &self,
        _vid: ViewId,
        _cx: &mut Context,
        _nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        // XXX: how does accesskit handle menu commands?
        None
    }
}

impl<V, F> private::Sealed for Command<V, F> {}

pub trait DynCommandBase {
    fn exec(&self);
    fn name(&self) -> Arc<str>;
    fn key(&self) -> Option<HotKey>;
}

pub trait CommandBase: DynCommandBase + Clone {}

impl<C: DynCommandBase + Clone> CommandBase for C {}

pub trait CommandTuple: Clone {
    fn foreach_cmd<F: FnMut(&dyn DynCommandBase)>(&self, f: &mut F);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        false
    } // satisfy clippy
}

impl<A: CommandBase> CommandTuple for (A,) {
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
    }
    fn len(&self) -> usize {
        1
    }
}

impl<A: CommandBase, B: CommandBase> CommandTuple for (A, B) {
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
    }
    fn len(&self) -> usize {
        2
    }
}

impl<A: CommandBase, B: CommandBase, C: CommandBase> CommandTuple for (A, B, C) {
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
    }
    fn len(&self) -> usize {
        3
    }
}

impl<A: CommandBase, B: CommandBase, C: CommandBase, D: CommandBase> CommandTuple for (A, B, C, D) {
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
    }
    fn len(&self) -> usize {
        4
    }
}

impl<A: CommandBase, B: CommandBase, C: CommandBase, D: CommandBase, E: CommandBase> CommandTuple
    for (A, B, C, D, E)
{
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
    }
    fn len(&self) -> usize {
        5
    }
}

impl<A: CommandBase, B: CommandBase, C: CommandBase, D: CommandBase, E: CommandBase, F: CommandBase>
    CommandTuple for (A, B, C, D, E, F)
{
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
        f(&self.5);
    }
    fn len(&self) -> usize {
        6
    }
}

impl<
    A: CommandBase,
    B: CommandBase,
    C: CommandBase,
    D: CommandBase,
    E: CommandBase,
    F: CommandBase,
    G: CommandBase,
> CommandTuple for (A, B, C, D, E, F, G)
{
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
        f(&self.5);
        f(&self.6);
    }
    fn len(&self) -> usize {
        7
    }
}

impl<
    A: CommandBase,
    B: CommandBase,
    C: CommandBase,
    D: CommandBase,
    E: CommandBase,
    F: CommandBase,
    G: CommandBase,
    H: CommandBase,
> CommandTuple for (A, B, C, D, E, F, G, H)
{
    fn foreach_cmd<FN: FnMut(&dyn DynCommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
        f(&self.5);
        f(&self.6);
        f(&self.7);
    }
    fn len(&self) -> usize {
        8
    }
}

#[derive(Clone)]
pub struct CommandGroup<V, C> {
    child: V,
    cmds: C,
}

impl<V, C> CommandGroup<V, C>
where
    V: View,
    C: CommandTuple,
{
    pub fn new(v: V, cmds: C) -> Self {
        Self { child: v, cmds }
    }
}

impl<V, C> DynView for CommandGroup<V, C>
where
    V: View,
    C: CommandTuple + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Command(name) = &event {
            self.cmds.foreach_cmd(&mut |cmd| {
                if cmd.name() == *name {
                    cmd.exec();
                }
            });
        }
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
        self.cmds.foreach_cmd(&mut |cmd| {
            cmds.push(CommandInfo {
                path: cmd.name(),
                key: cmd.key(),
            })
        });
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        self.child.access(cid, cx, nodes)
    }
}

impl<V, C> private::Sealed for CommandGroup<V, C> {}

#[derive(Clone)]
pub struct NullCommand {
    name: Arc<str>,
    key: Option<HotKey>,
}

/// Specifies a menu command.
pub fn command(name: &str) -> NullCommand {
    NullCommand {
        name: name.into(),
        key: None,
    }
}

impl DynCommandBase for NullCommand {
    fn exec(&self) {}
    fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    fn key(&self) -> Option<HotKey> {
        None
    }
}

impl NullCommand {
    /// Adds a hotkey to the menu command.
    pub fn hotkey(self, key: HotKey) -> Self {
        Self {
            name: self.name,
            key: Some(key),
        }
    }
    /// Adds an action to the menu command.
    pub fn action<F: Fn()>(self, func: F) -> Command2<F> {
        Command2 {
            name: self.name,
            key: self.key,
            func,
        }
    }
}

#[derive(Clone)]
pub struct Command2<F> {
    name: Arc<str>,
    key: Option<HotKey>,
    func: F,
}

impl<F> DynCommandBase for Command2<F>
where
    F: Fn(),
{
    fn exec(&self) {
        (self.func)();
    }
    fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    fn key(&self) -> Option<HotKey> {
        self.key
    }
}

impl<F> Command2<F>
where
    F: Fn(),
{
    /// Adds a hotkey to the menu command.
    pub fn hotkey(self, key: HotKey) -> Self {
        Self {
            name: self.name,
            key: Some(key),
            func: self.func,
        }
    }
}

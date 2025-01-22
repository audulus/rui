use crate::*;
use std::any::Any;

pub struct Command<V, F> {
    child: V,
    name: String,
    key: Option<HotKey>,
    func: F,
}

impl<V, F> Command<V, F>
where
    V: View,
    F: Fn(&mut Context) + 'static,
{
    pub fn new(v: V, name: String, key: Option<HotKey>, f: F) -> Self {
        Self {
            child: v,
            name,
            key,
            func: f,
        }
    }
}

impl<V, F> View for Command<V, F>
where
    V: View,
    F: Fn(&mut Context) + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Command(name) = &event {
            if *name == self.name {
                (self.func)(cx);
            }
        }
        path.push(0);
        self.child.process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(0);
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = self.child.layout(path, args);
        path.pop();
        sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let id = self.child.hittest(path, pt, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
        cmds.push(CommandInfo {
            path: self.name.clone(),
            key: self.key,
        })
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        _path: &mut IdPath,
        _cx: &mut Context,
        _nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        // XXX: how does accesskit handle menu commands?
        None
    }
}

impl<V, F> private::Sealed for Command<V, F> {}

pub trait CommandBase {
    fn exec(&self);
    fn name(&self) -> String;
    fn key(&self) -> Option<HotKey>;
}

pub trait CommandTuple {
    fn foreach_cmd<F: FnMut(&dyn CommandBase)>(&self, f: &mut F);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        false
    } // satisfy clippy
}

impl<A: CommandBase> CommandTuple for (A,) {
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
        f(&self.0);
    }
    fn len(&self) -> usize {
        1
    }
}

impl<A: CommandBase, B: CommandBase> CommandTuple for (A, B) {
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
    }
    fn len(&self) -> usize {
        2
    }
}

impl<A: CommandBase, B: CommandBase, C: CommandBase> CommandTuple for (A, B, C) {
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
    }
    fn len(&self) -> usize {
        3
    }
}

impl<A: CommandBase, B: CommandBase, C: CommandBase, D: CommandBase> CommandTuple for (A, B, C, D) {
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
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
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
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

impl<
        A: CommandBase,
        B: CommandBase,
        C: CommandBase,
        D: CommandBase,
        E: CommandBase,
        F: CommandBase,
    > CommandTuple for (A, B, C, D, E, F)
{
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
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
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
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
    fn foreach_cmd<FN: FnMut(&dyn CommandBase)>(&self, f: &mut FN) {
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

impl<V, C> View for CommandGroup<V, C>
where
    V: View,
    C: CommandTuple + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
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
        path.push(0);
        self.child.process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(0);
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = self.child.layout(path, args);
        path.pop();
        sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let id = self.child.hittest(path, pt, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
        self.cmds.foreach_cmd(&mut |cmd| {
            cmds.push(CommandInfo {
                path: cmd.name(),
                key: cmd.key(),
            })
        });
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<V, C> private::Sealed for CommandGroup<V, C> {}

pub struct NullCommand {
    name: String,
    key: Option<HotKey>,
}

/// Specifies a menu command.
pub fn command(name: &str) -> NullCommand {
    NullCommand {
        name: name.into(),
        key: None,
    }
}

impl CommandBase for NullCommand {
    fn exec(&self) {}
    fn name(&self) -> String {
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

pub struct Command2<F: Fn()> {
    name: String,
    key: Option<HotKey>,
    func: F,
}

impl<F> CommandBase for Command2<F>
where
    F: Fn(),
{
    fn exec(&self) {
        (self.func)();
    }
    fn name(&self) -> String {
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

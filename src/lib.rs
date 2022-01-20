use std::any::{Any, TypeId};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Stack {
    HStack,
    VStack,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum Command {
    Button(String),
    Text(String),
    Begin(Stack),
    End,
}

enum Event {
    Draw,
    Process,
    TapButton(String),
}

pub struct Gui {
    commands: Vec<Command>,
    event: Event,
    state: Box<dyn Any>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            commands: vec![],
            event: Event::Draw,
            state: Box::new(0),
        }
    }

    pub fn button(&mut self, name: &str) -> bool {
        match &self.event {
            Event::Draw => {
                self.commands.push(Command::Button(String::from(name)));
                false
            }
            Event::Process => false,
            Event::TapButton(n) => n == name,
            _ => false,
        }
    }

    pub fn text(&mut self, name: &str) {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::Text(String::from(name)));
            }
            Event::Process => {}
            _ => {}
        }
    }

    fn begin_hstack(&mut self) {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::Begin(Stack::HStack));
            }
            Event::Process => {}
            _ => {}
        }
    }

    fn end_hstack(&mut self) {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::End);
            }
            Event::Process => {}
            _ => {}
        }
    }

    pub fn hstack<F: FnOnce(&mut Gui)>(&mut self, f: F) {
        self.begin_hstack();
        f(self);
        self.end_hstack();
    }

    pub fn state<F: FnOnce(&mut Gui, &mut S), S: Default + Any + Clone>(&mut self, f: F) {
        if let Some(mut s) = self.state.downcast_mut::<S>() {
            let mut g = Self::new();
            f(&mut g, &mut s);
        } else {
            let mut st = S::default();
            f(self, &mut st);
            self.state = Box::new(st);
        }
    }
}

pub fn gui<F: Fn(&mut Gui)>(f: F) {
    let mut gui = Gui::new();
    f(&mut gui);
}

pub struct Context {
    state: usize
}

impl Context {
    pub fn get(&self) -> usize {
        self.state
    }

    pub fn set(&mut self, value: usize) {
        self.state = value
    }
}

pub trait View { }

pub struct EmptyView { }

impl View for EmptyView { }

pub struct State<'a, V: View> { 
    func: Box<dyn Fn(&Context) -> V + 'a>
}

impl<'a, V> View for State<'a, V> where V: View { }

pub fn state<'a, V: View, F: Fn(&Context) -> V + 'a>(f: F) -> State<'a, V> {
    State{func: Box::new(f)}
}

pub struct Button<'a> {
    text: String,
    func: Box<dyn Fn(&mut Context) + 'a>
}

impl<'a> View for Button<'a> { }

pub fn button<'a, F: Fn(&mut Context) + 'a>(name: &str, f: F) -> Button<'a> {
    Button{text: String::from(name), func: Box::new(f)}
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn describe_gui() {
        let mut gui = Gui::new();

        gui.hstack(|gui| {
            if gui.button("click me!") {
                println!("clicked!")
            }
            gui.text("Hello world!");
        })
    }

    #[test]
    fn test_gui() {
        gui(|gui| {
            gui.hstack(|gui| {
                if gui.button("click me!") {
                    println!("clicked!")
                }
                gui.text("Hello world!");
            })
        })
    }

    #[test]
    fn test_counter() {
        gui(|gui| {
            gui.state(|gui, state: &mut usize| {
                gui.hstack(|gui| {
                    if gui.button("click me!") {
                        println!("clicked!");
                        *state += 1;
                    }
                    gui.text(format!("{:?}", state).as_str());
                })
            })
        })
    }

    #[test]
    fn test_button() {
        let _ = button("click me", |cx| {
            println!("clicked!");
        });
    }

    
    #[test]
    fn test_state() {
        let _ = state(|_cx: &Context| {
            EmptyView{}
        });
    }

    #[test]
    fn test_state2() {
        let _ = state(|cx| {
            button(format!("{:?}", cx.get()).as_str(), |cx|{
                cx.set( cx.get() + 1);
            })
        });
    }
}

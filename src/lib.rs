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

trait View { }

struct State<S: Default> { 
    func: Box<dyn Fn(&mut S) -> Box<dyn View> >
}

impl<S> View for State<S> where S: Default { }

pub fn state<F: Fn(&mut S) -> Box<dyn View> + 'static, S: Default + 'static>(f: F) -> Box<dyn View> {
    Box::new(State{func: Box::new(f)})
}

pub struct Button<'a> {
    text: String,
    func: Box<dyn Fn() + 'a>
}

impl<'a> View for Button<'a> { }

pub fn button<'a, F: Fn() + 'a>(name: &str, f: F) -> Box<Button<'a>> {
    Box::new(Button{text: String::from(name), func: Box::new(f)})
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
        let _ = button("click me", || {
            println!("clicked!");
        });
    }

    /*
    #[test]
    fn test_counter2() {
        let _ = state(|state: &mut usize| {
            button(format!("{:?}", state).as_str(), ||{
                *state += 1;
            })
        });
    }
    */
}


#[derive(Clone, Copy, Eq, PartialEq)]
enum Stack {
    HStack,
    VStack
}

#[derive(Clone, Eq, PartialEq)]
enum Command {
    Button(String),
    Text(String),
    Begin(Stack),
    End
}

enum Event {
    Draw,
    Process,
}

pub struct Gui {
    commands: Vec<Command>,
    event: Event
}

impl Gui {

    pub fn new() -> Self {
        Self {
            commands: vec![],
            event: Event::Draw
        }
    }

    pub fn button(&mut self, name: &str) -> bool {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::Button(String::from(name)));
                false
            }
            Event::Process => {
                false
            }
        }
    }

    pub fn text(&mut self, name: &str) {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::Text(String::from(name)));
            }
            Event::Process => { }
        }
    }

    fn begin_hstack(&mut self) {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::Begin(Stack::HStack));
            }
            Event::Process => { }
        }
    }

    fn end_hstack(&mut self) {
        match self.event {
            Event::Draw => {
                self.commands.push(Command::End);
            }
            Event::Process => { }
        }
    }

    pub fn hstack<F : FnOnce(&mut Gui)>(&mut self, f: F) {
        self.begin_hstack();
        f(self);
        self.end_hstack();
    }

    // XXX: how do we generically store and retrieve state?
    pub fn state<F : FnOnce(&mut Gui, &mut S), S : Default> (&mut self, f: F) {
        let mut state = S::default();
        f(self, &mut state);
    }
}

pub fn gui<F : Fn(&mut Gui)>(f: F) {
    let mut gui = Gui::new();
    f(&mut gui);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn describe_gui() {
        
        let mut gui = Gui::new();

        gui.hstack(|gui|{
            if gui.button("click me!") {
                println!("clicked!")
            }
            gui.text("Hello world!");
        })
    }

    #[test]
    fn test_gui() {
        gui(|gui| {
            gui.hstack(|gui|{
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
                gui.hstack(|gui|{
                    if gui.button("click me!") {
                        println!("clicked!");
                        *state += 1;
                    }
                    gui.text(format!("{:?}", state).as_str());
                })
            })
        })
    }

}

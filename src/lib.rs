
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

enum State {
    Draw,
    Process,
}

pub struct Gui {
    commands: Vec<Command>,
    state: State
}

impl Gui {

    pub fn new() -> Self {
        Self {
            commands: vec![],
            state: State::Draw
        }
    }

    pub fn button(&mut self, name: &str) -> bool {
        match self.state {
            State::Draw => {
                self.commands.push(Command::Button(String::from(name)));
                false
            }
            State::Process => {
                false
            }
        }
    }

    pub fn text(&mut self, name: &str) {
        match self.state {
            State::Draw => {
                self.commands.push(Command::Text(String::from(name)));
            }
            State::Process => { }
        }
    }

    fn begin_hstack(&mut self) {
        match self.state {
            State::Draw => {
                self.commands.push(Command::Begin(Stack::HStack));
            }
            State::Process => { }
        }
    }

    fn end_hstack(&mut self) {
        match self.state {
            State::Draw => {
                self.commands.push(Command::End);
            }
            State::Process => { }
        }
    }

    pub fn hstack<F : FnOnce(&mut Gui)>(&mut self, f: F) {
        self.begin_hstack();
        f(self);
        self.end_hstack();
    }
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

}

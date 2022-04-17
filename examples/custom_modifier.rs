use rui::*;

#[derive(Clone, Copy)]
enum MyControlType {
    Chill,
    Agro
}

impl Default for MyControlType {
    fn default() -> Self {
        Self::Chill
    }
}

trait MyMods: View + Sized {
    fn agro(self) -> SetenvView<Self, MyControlType>;
}

fn my_control() -> impl MyMods {
    env(|t, _| {
        circle().color( match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT
        })
    }).env_mod::<MyControlType>()
}

impl<V: View> MyMods for SetenvView<V, MyControlType> {
    fn agro(self) -> SetenvView<Self, MyControlType> {
        self.env(MyControlType::Agro)
    }
}

fn main() {
    rui( 
        vstack((
            my_control().padding(Auto),
            my_control().agro().padding(Auto)
        ))
    )
}
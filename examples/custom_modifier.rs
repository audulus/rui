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
    fn agro(self) -> ModView<AnyView, MyControlType>;
}

fn my_control() -> impl MyMods {
    modview(env(|t, _| {
        circle().color( match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT
        })
    }))
}

impl<V: View> MyMods for ModView<V, MyControlType> {
    fn agro(self) -> ModView<AnyView, MyControlType> {
        modview(any_view(self.env(MyControlType::Agro)))
    }
}

fn main() {
    rui( 
        vstack((
            my_control(),
            my_control().agro()
        ))
    )
}
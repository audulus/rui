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

fn my_control() -> ModView<impl View, MyControlType> {
    modview(env(|t, _| {
        circle().color( match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT
        })
    }))
}

trait MyMods: View + Sized {
    fn agro(self) -> ModView<AnyView, MyControlType>;
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
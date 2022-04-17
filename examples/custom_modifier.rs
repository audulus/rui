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
    fn agro(self) -> Self;
}

fn my_control() -> impl MyMods {
    modview(|t, _| {
        circle().color( match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT
        })
    })
}

impl<V, F> MyMods for ModView<MyControlType, F>
where
    V: View,
    F: Fn(&MyControlType, &mut Context) -> V + 'static,
{
    fn agro(self) -> Self {
        ModView { func: self.func, value: MyControlType::Agro }
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
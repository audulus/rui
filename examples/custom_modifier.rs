use rui::*;

#[derive(Clone, Copy)]
enum MyControlType {
    Chill,
    Agro,
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
        circle().color(match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT,
        })
    })
}

impl<F> MyMods for ModView<MyControlType, F>
where
    ModView<MyControlType, F>: View,
{
    fn agro(self) -> Self {
        ModView {
            func: self.func,
            value: MyControlType::Agro,
        }
    }
}

fn main() {
    vstack((
        my_control().padding(Auto),
        my_control().agro().padding(Auto),
    ))
    .run()
}

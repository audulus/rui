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

fn my_control() -> impl View {
    env(|t, _| {
        circle().color(match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT,
        })
    })
}

fn main() {
    rui(vstack((
        my_control(),
        my_control().env(MyControlType::Agro),
    )))
}

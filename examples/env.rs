use rui::*;

#[derive(Clone, Copy)]
enum MyControlType {
    Chill,
    Agro
}

fn my_control() -> impl View {
    env(|| MyControlType::Chill, |t, _| {
        circle().color( match t {
            MyControlType::Chill => AZURE_HIGHLIGHT,
            MyControlType::Agro => RED_HIGHLIGHT
        })
    })
}

fn main() {
    rui( 
        vstack((
            my_control().env(MyControlType::Chill),
            my_control().env(MyControlType::Agro)
        ))
    )
}
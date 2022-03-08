use rui::*;

fn main() {
    rui(hstack((
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto)
            .command("File:Command One", || println!("command one") ),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
            .command("Edit:Command Two", || println!("command two") )
    )));
}

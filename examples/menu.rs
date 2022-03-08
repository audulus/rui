use rui::*;

fn main() {
    rui(hstack((
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto)
            .command("menu:About", || println!("about") )
            .command("menu:Quit", || println!("quit") )
            .command("File:New", || println!("new") ),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
            .command("Edit:Two", || println!("two") )
            .command("Edit:Three", || println!("three") )
    )));
}

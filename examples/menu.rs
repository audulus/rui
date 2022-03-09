use rui::*;

fn main() {
    rui(hstack((
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto)
            .command("File:New", Some(KeyCode::KeyN), || println!("new") ),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
            .command("Edit:Two", None, || println!("two") )
            .command("Edit:Three", None, || println!("three") )
            .command("Custom:Submenu:One", None, || println!("submenu one") )
            .command("Custom:Submenu:Two", None, || println!("submenu two") )
            .command_group((
                command("Custom 2:Four", || println!("four") )
                    .hotkey(KeyCode::KeyF),
            ))
    )));
}

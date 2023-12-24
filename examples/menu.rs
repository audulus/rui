use rui::*;

// Requires tao instead of winit for menus.
// Run with: cargo run --example menu --no-default-features --features tao

fn main() {
    hstack((
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto)
            .command("File:New", Some(HotKey::KeyN), |_| println!("new")),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
            .command("Edit:Two", None, |_| println!("two"))
            .command("Edit:Three", None, |_| println!("three"))
            .command("Custom:Submenu:One", None, |_| println!("submenu one"))
            .command("Custom:Submenu:Two", None, |_| println!("submenu two"))
            .command_group((command("Custom 2:Four")
                .action(|| println!("four"))
                .hotkey(HotKey::KeyF),)),
    ))
    .run()
}

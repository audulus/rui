use rui::*;

fn main() {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    rui(vstack((
        state(
            move || lorem.to_string().clone(),
            |state| text_editor(state).padding(Auto),
        )
        .background(
            rectangle()
                .color(BUTTON_BACKGROUND_COLOR)
                .corner_radius(5.0),
        )
        .padding(Auto),
        state(
            move || lorem.to_string().clone(),
            |state| text_editor(state).padding(Auto),
        )
        .background(
            rectangle()
                .color(BUTTON_BACKGROUND_COLOR)
                .corner_radius(5.0),
        )
        .padding(Auto),
    )));
}

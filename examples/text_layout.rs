use rui::*;

fn main() {
    rui(canvas(|rect, vger| {

        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        let paint = vger.color_paint(vger::Color::MAGENTA);

        vger.translate([32.0, 256.0]);

        let bounds = vger.text_bounds(lorem, 18, Some(448.0));

        vger.stroke_rect(
            bounds.origin,
            bounds.max(),
            10.0,
            4.0,
            paint,
        );

        vger.text(lorem, 18, vger::Color::CYAN, Some(448.0));
    }));
}

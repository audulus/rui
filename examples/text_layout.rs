use rui::*;

fn main() {
    canvas(|_cx, rect, vger| {

        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        let paint = vger.color_paint(vger::Color::MAGENTA.alpha(0.2));

        vger.translate([0.0, rect.height()]);

        let font_size = 24;
        let break_width = Some(rect.width());

        let bounds = vger.text_bounds(lorem, font_size, break_width);

        vger.stroke_rect(
            bounds.origin,
            bounds.max(),
            10.0,
            4.0,
            paint,
        );

        let rects = vger.glyph_positions(lorem, font_size, break_width);

        let glyph_rect_paint = vger.color_paint(vger::Color::MAGENTA.alpha(0.1));

        for rect in rects {
            vger.fill_rect(rect, 0.0, glyph_rect_paint);
        }

        let lines = vger.line_metrics(lorem, font_size, break_width);

        let line_rect_paint = vger.color_paint(RED_HIGHLIGHT.alpha(0.1));

        for line in lines {
            vger.fill_rect(line.bounds, 0.0, line_rect_paint);
        }

        vger.text(lorem, font_size, TEXT_COLOR, break_width);

    }).padding(Auto).run()
}

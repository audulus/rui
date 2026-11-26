//! The messages page: a `list` of message cards whose bodies wrap at the card's right padding,
//! so a card is as tall as its text.

use std::sync::Arc;

use rui::*;

use super::style::{Surface, color, gap, label, paint_centered_text};
use crate::dash::{self, Message};

/// Returns the feed of `messages`.
pub fn view(messages: Arc<Vec<Message>>) -> impl View {
    let count = messages.len();
    list((0..count).collect(), move |&i| {
        let below = if i + 1 < count {
            dash::MESSAGE_GAP
        } else {
            0.0
        };
        vstack((card(&messages[i]), gap(0.0, below)))
    })
    .padding(dash::CONTENT_PADDING)
}

/// The card of one message.
fn card(message: &Message) -> impl View + use<> {
    let (down, across) = dash::MESSAGE_PADDING;
    let head = hstack((
        gap(across, dash::MESSAGE_LINE),
        avatar(&message.initials, message.avatar),
        gap(dash::MESSAGE_TEXT_X - across - dash::MESSAGE_AVATAR, 0.0),
        label(&message.author, dash::FONT_SIZE, dash::TEXT),
        gap(8.0, 0.0),
        label(&message.time, 13.0, dash::MUTED),
        spacer(),
    ));
    let body = hstack((
        gap(dash::MESSAGE_TEXT_X, 0.0),
        label(&message.body, dash::FONT_SIZE, dash::TEXT)
            .wrap()
            .flex(),
        gap(across, 0.0),
    ));
    vstack((gap(0.0, down), head, gap(0.0, 4.0), body, gap(0.0, down)))
        .background(Surface::CARD.view())
}

/// The avatar of a message: a circle whose top is the top of the author's line, which it
/// overhangs downwards.
fn avatar(initials: &str, hex: u32) -> impl View + use<> {
    let initials = initials.to_owned();
    canvas(move |_, rect, vger| {
        let r = dash::MESSAGE_AVATAR / 2.0;
        let center = LocalPoint::new(r, rect.height() - r);
        let paint = vger.color_paint(color(hex));
        vger.fill_circle(center, r, paint);
        paint_centered_text(vger, &initials, 13, 0xffffff, center);
    })
    .size([dash::MESSAGE_AVATAR, dash::MESSAGE_LINE])
}

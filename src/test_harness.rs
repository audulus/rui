use crate::*;

/// A headless test harness for simulating input sequences on views.
///
/// Handles layout, event processing, and state access without
/// needing a GPU or window.
///
/// # Example
/// ```ignore
/// let ui = state(|| 0, |count, _| {
///     rectangle().size([100.0, 50.0]).tap(move |cx| cx[count] += 1)
/// });
/// let mut h = TestHarness::new(&ui, [100.0, 50.0]);
/// h.tap([50.0, 25.0]);
/// let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
/// assert_eq!(h.cx[s], 1);
/// ```
pub(crate) struct TestHarness<'a, V: View> {
    view: &'a V,
    pub cx: Context,
    sz: LocalSize,
}

impl<'a, V: View> TestHarness<'a, V> {
    /// Create a new harness for the given view and window size.
    pub fn new(view: &'a V, sz: impl Into<LocalSize>) -> Self {
        let sz = sz.into();
        let mut harness = Self {
            view,
            cx: Context::new(),
            sz,
        };
        harness.layout();
        harness
    }

    /// Run layout, computing sizes and positions for all views.
    pub fn layout(&mut self) {
        let mut path = vec![0];
        self.view.layout(
            &mut path,
            &mut LayoutArgs {
                sz: self.sz,
                cx: &mut self.cx,
                // Approximate text bounds: ~half of font size per char.
                text_bounds: &mut |text, size, _max_width| {
                    let w = text.len() as f32 * size as f32 * 0.5;
                    let h = size as f32;
                    LocalRect::new(LocalPoint::zero(), [w, h].into())
                },
            },
        );
    }

    /// Send a single event to the view tree (no re-layout).
    fn send_event(&mut self, event: &Event) {
        let mut actions = vec![];
        let mut path = vec![0];
        self.view
            .process(event, &mut path, &mut self.cx, &mut actions);
    }

    /// Send a single event to the view tree, then re-layout.
    pub fn event(&mut self, event: &Event) {
        self.send_event(event);
        self.layout();
    }

    /// Simulate a tap (touch begin + end) at the given position.
    pub fn tap(&mut self, position: impl Into<LocalPoint>) {
        let position = position.into();
        self.send_event(&Event::TouchBegin { id: 0, position });
        self.send_event(&Event::TouchEnd { id: 0, position });
        self.layout();
    }

    /// Simulate a drag from one position to another.
    pub fn drag(&mut self, from: impl Into<LocalPoint>, to: impl Into<LocalPoint>) {
        let from = from.into();
        let to = to.into();
        let delta = to - from;
        self.send_event(&Event::TouchBegin {
            id: 0,
            position: from,
        });
        self.send_event(&Event::TouchMove {
            id: 0,
            position: to,
            delta,
        });
        self.send_event(&Event::TouchEnd {
            id: 0,
            position: to,
        });
        self.layout();
    }

    /// Simulate a key press.
    pub fn key(&mut self, k: Key) {
        self.send_event(&Event::Key(k));
        self.layout();
    }

    /// Simulate typing a string (sends Character key events).
    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            if c == ' ' {
                self.send_event(&Event::Key(Key::Space));
            } else {
                self.send_event(&Event::Key(Key::Character(c)));
            }
        }
        self.layout();
    }

    /// Set keyboard modifier state (shift, ctrl, etc).
    pub fn set_key_mods(&mut self, mods: KeyboardModifiers) {
        self.cx.key_mods = mods;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Basic tap: rectangle + tap increments counter ---

    #[test]
    fn test_tap_increments() {
        let ui = state(
            || 0i32,
            |count, _| {
                rectangle()
                    .size([100.0, 50.0])
                    .tap(move |cx| cx[count] += 1)
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], 0);

        h.tap([50.0, 25.0]);
        assert_eq!(h.cx[s], 1);

        h.tap([50.0, 25.0]);
        assert_eq!(h.cx[s], 2);
    }

    // --- Repeated taps ---

    #[test]
    fn test_repeated_taps() {
        let ui = state(
            || 0i32,
            |count, _| {
                rectangle()
                    .size([100.0, 50.0])
                    .tap(move |cx| cx[count] += 1)
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));

        for expected in 1..=10 {
            h.tap([50.0, 25.0]);
            assert_eq!(h.cx[s], expected);
        }
    }

    // --- Toggle: tap flips boolean state ---

    #[test]
    fn test_toggle_tap() {
        let ui = state(|| false, |s, _| toggle(s));

        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        let s = StateHandle::<bool>::new(h.cx.view_id(&vec![0]));
        assert!(!h.cx[s]);

        h.tap([10.0, 10.0]);
        assert!(h.cx[s]);

        h.tap([10.0, 10.0]);
        assert!(!h.cx[s]);
    }

    // --- Knob: drag changes value ---

    #[test]
    fn test_knob_drag() {
        let ui = state(|| 0.5f32, |s, _| knob(s));

        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let s = StateHandle::<f32>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], 0.5);

        // Drag rightward should increase value.
        h.drag([50.0, 50.0], [100.0, 50.0]);
        assert!(h.cx[s] > 0.5);
    }

    // --- Two tappable views in an hstack ---

    #[test]
    fn test_two_tappable_views() {
        let ui = state(
            || (0i32, 0i32),
            |counts, _| {
                hstack((
                    rectangle()
                        .size([100.0, 50.0])
                        .tap(move |cx| cx[counts].0 += 1),
                    rectangle()
                        .size([100.0, 50.0])
                        .tap(move |cx| cx[counts].1 += 1),
                ))
            },
        );

        let mut h = TestHarness::new(&ui, [200.0, 50.0]);
        let s = StateHandle::<(i32, i32)>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], (0, 0));

        // Tap left half.
        h.tap([50.0, 25.0]);
        assert_eq!(h.cx[s].0, 1);
        assert_eq!(h.cx[s].1, 0);

        // Tap right half.
        h.tap([150.0, 25.0]);
        assert_eq!(h.cx[s].0, 1);
        assert_eq!(h.cx[s].1, 1);
    }

    // --- Drag sequence records gesture states ---

    #[test]
    fn test_drag_gesture_states() {
        let ui = state(
            || vec![],
            |states, _| {
                rectangle()
                    .size([100.0, 100.0])
                    .drag(move |cx, _delta, gesture_state, _| cx[states].push(gesture_state))
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let s = StateHandle::<Vec<GestureState>>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], vec![]);

        h.drag([50.0, 50.0], [80.0, 50.0]);
        assert_eq!(
            h.cx[s],
            vec![
                GestureState::Began,
                GestureState::Changed,
                GestureState::Ended,
            ]
        );
    }

    // --- Tap outside view doesn't trigger ---

    #[test]
    fn test_tap_miss() {
        let ui = state(
            || 0i32,
            |count, _| {
                rectangle()
                    .size([50.0, 50.0])
                    .tap(move |cx| cx[count] += 1)
            },
        );

        let mut h = TestHarness::new(&ui, [200.0, 200.0]);
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));

        // Tap outside the 50x50 rectangle.
        h.tap([150.0, 150.0]);
        assert_eq!(h.cx[s], 0);

        // Tap inside.
        h.tap([25.0, 25.0]);
        assert_eq!(h.cx[s], 1);
    }

    // --- Key events ---

    #[test]
    fn test_key_events() {
        let ui = state(
            || Vec::<Key>::new(),
            |keys, _| {
                rectangle().size([100.0, 100.0]).key(move |cx, k| {
                    cx[keys].push(k);
                })
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let s = StateHandle::<Vec<Key>>::new(h.cx.view_id(&vec![0]));

        h.key(Key::ArrowLeft);
        h.key(Key::Character('a'));
        h.key(Key::Space);

        assert_eq!(
            h.cx[s],
            vec![Key::ArrowLeft, Key::Character('a'), Key::Space]
        );
    }

    // --- type_text helper ---

    #[test]
    fn test_type_text() {
        let ui = state(
            || Vec::<Key>::new(),
            |keys, _| {
                rectangle().size([100.0, 100.0]).key(move |cx, k| {
                    cx[keys].push(k);
                })
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let s = StateHandle::<Vec<Key>>::new(h.cx.view_id(&vec![0]));

        h.type_text("hi ");

        assert_eq!(
            h.cx[s],
            vec![Key::Character('h'), Key::Character('i'), Key::Space]
        );
    }

    // --- Modifier state ---

    #[test]
    fn test_key_mods() {
        let ui = state(
            || false,
            |shift_was_down, _| {
                rectangle().size([100.0, 100.0]).key(move |cx, _k| {
                    cx[shift_was_down] = cx.key_mods.shift;
                })
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let s = StateHandle::<bool>::new(h.cx.view_id(&vec![0]));

        h.set_key_mods(KeyboardModifiers {
            shift: true,
            ..Default::default()
        });
        h.key(Key::ArrowRight);
        assert!(h.cx[s]);

        h.set_key_mods(KeyboardModifiers::default());
        h.key(Key::ArrowRight);
        assert!(!h.cx[s]);
    }

    // --- Vertical list tap ---

    #[test]
    fn test_vlist_tap() {
        let ui = state(
            || None::<i32>,
            |selected, _| {
                list(vec![0, 1, 2], move |id| {
                    let id = *id;
                    rectangle()
                        .size([100.0, 30.0])
                        .tap(move |cx| cx[selected] = Some(id))
                })
            },
        );

        let mut h = TestHarness::new(&ui, [100.0, 90.0]);
        let s = StateHandle::<Option<i32>>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], None);

        // List is 100x90, items stacked vertically each 30px tall.
        // In rui's coordinate system, y=0 is bottom, items laid out top-to-bottom
        // means item 0 is at y=60..90, item 1 at y=30..60, item 2 at y=0..30.
        h.tap([50.0, 75.0]); // top item (id=0)
        assert_eq!(h.cx[s], Some(0));

        h.tap([50.0, 15.0]); // bottom item (id=2)
        assert_eq!(h.cx[s], Some(2));
    }

    // --- Cond with state ---

    #[test]
    fn test_cond_switches() {
        let ui = state(
            || false,
            |flag, cx| {
                vstack((
                    cond(
                        cx[flag],
                        rectangle().size([200.0, 50.0]),
                        rectangle().size([100.0, 50.0]),
                    ),
                    rectangle()
                        .size([100.0, 30.0])
                        .tap(move |cx| cx[flag] = !cx[flag]),
                ))
            },
        );

        let mut h = TestHarness::new(&ui, [200.0, 80.0]);
        let s = StateHandle::<bool>::new(h.cx.view_id(&vec![0]));
        assert!(!h.cx[s]);

        // Tap the bottom rectangle (the "button").
        // vstack: top is cond (50px), bottom is tap rect (30px).
        // In y-up coords: tap rect is at y=0..30, cond is at y=30..80.
        h.tap([50.0, 15.0]);
        assert!(h.cx[s]);

        // Tap again to toggle back.
        h.tap([50.0, 15.0]);
        assert!(!h.cx[s]);
    }

    // --- Tap action produces correct action type ---

    #[test]
    fn test_tap_action() {
        #[derive(Clone, Debug, PartialEq)]
        enum Action {
            Clicked,
        }

        let ui = rectangle().size([100.0, 100.0]).tap_a(Action::Clicked);

        let mut h = TestHarness::new(&ui, [100.0, 100.0]);

        // Process events manually to capture actions.
        let mut actions = vec![];
        let mut path = vec![0];
        let events = [
            Event::TouchBegin {
                id: 0,
                position: [50.0, 50.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [50.0, 50.0].into(),
            },
        ];
        for event in &events {
            ui.process(event, &mut path, &mut h.cx, &mut actions);
        }

        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0].downcast_ref::<Action>(),
            Some(&Action::Clicked)
        );
    }
}

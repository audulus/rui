// Taken from wnit_input_helper crate
// The MIT License (MIT)
// Copyright (c) 2018 Lucas Kent
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{Key, PhysicalKey};

#[derive(Clone)]
pub struct CurrentInput {
    pub mouse_actions: Vec<MouseAction>,
    pub key_actions: Vec<KeyAction>,
    pub scancode_actions: Vec<ScanCodeAction>,
    pub key_held: Vec<Key>,
    pub scancode_held: Vec<PhysicalKey>, // some scan codes are higher than 255 so using an array may be dangerous
    pub mouse_held: [bool; 255],
    pub cursor_point: Option<(f32, f32)>,
    pub cursor_point_prev: Option<(f32, f32)>,
    pub mouse_diff: Option<(f32, f32)>,
    pub y_scroll_diff: f32,
    pub x_scroll_diff: f32,
    pub text: Vec<Key>,
}

impl CurrentInput {
    pub fn new() -> CurrentInput {
        CurrentInput {
            mouse_actions: vec![],
            key_actions: vec![],
            scancode_actions: vec![],
            key_held: vec![],
            scancode_held: vec![],
            mouse_held: [false; 255],
            cursor_point: None,
            cursor_point_prev: None,
            mouse_diff: None,
            y_scroll_diff: 0.0,
            x_scroll_diff: 0.0,
            text: vec![],
        }
    }

    pub fn step(&mut self) {
        self.mouse_actions.clear();
        self.key_actions.clear();
        self.scancode_actions.clear();
        self.cursor_point_prev = self.cursor_point;
        self.mouse_diff = None;
        self.y_scroll_diff = 0.0;
        self.x_scroll_diff = 0.0;
        self.text.clear();
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => match event.state {
                ElementState::Pressed => {
                    let logical_key = &event.logical_key;
                    if !self.key_held.contains(logical_key) {
                        self.key_actions
                            .push(KeyAction::Pressed(logical_key.clone()));
                    }

                    self.key_held.push(logical_key.clone());
                    self.key_actions
                        .push(KeyAction::PressedOs(logical_key.clone()));
                    self.text.push(logical_key.clone());

                    let physical_key = &event.physical_key;
                    if !self.scancode_held.contains(physical_key) {
                        self.scancode_actions
                            .push(ScanCodeAction::Pressed(*physical_key));
                        self.scancode_held.push(*physical_key);
                    }

                    self.scancode_actions
                        .push(ScanCodeAction::PressedOs(*physical_key));
                }
                ElementState::Released => {
                    let logical_key = &event.logical_key;
                    self.key_held.retain(|x| x != logical_key);
                    self.key_actions
                        .push(KeyAction::Released(logical_key.clone()));

                    let physical_key = &event.physical_key;
                    self.scancode_held.retain(|x| x != physical_key);
                    self.scancode_actions
                        .push(ScanCodeAction::Released(*physical_key));
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_point = Some((position.x as f32, position.y as f32));
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                let button_usize = mouse_button_to_int(button);
                self.mouse_held[button_usize] = true;
                self.mouse_actions.push(MouseAction::Pressed(*button));
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => {
                let button_usize = mouse_button_to_int(button);
                self.mouse_held[button_usize] = false;
                self.mouse_actions.push(MouseAction::Released(*button));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // I just took this from three-rs, no idea why this magic number was chosen ¯\_(ツ)_/¯
                const PIXELS_PER_LINE: f64 = 38.0;

                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.x_scroll_diff += x;
                        self.y_scroll_diff += y;
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.y_scroll_diff += (delta.y / PIXELS_PER_LINE) as f32;
                        self.x_scroll_diff += (delta.x / PIXELS_PER_LINE) as f32;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta, .. } = event {
            match self.mouse_diff {
                Some((x, y)) => self.mouse_diff = Some((x + delta.0 as f32, y + delta.1 as f32)),
                None => self.mouse_diff = Some((delta.0 as f32, delta.1 as f32)),
            }
        }
    }
}

#[derive(Clone)]
pub enum KeyAction {
    Pressed(Key),
    PressedOs(Key),
    Released(Key),
}

#[derive(Clone, PartialEq)]
pub enum ScanCodeAction {
    Pressed(PhysicalKey),
    PressedOs(PhysicalKey),
    Released(PhysicalKey),
}

#[derive(Clone)]
pub enum MouseAction {
    Pressed(MouseButton),
    Released(MouseButton),
}

pub fn mouse_button_to_int(button: &MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(byte) => 5 + *byte as usize,
    }
}

use std::fmt::{self, Debug};

use vulkan::winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, ModifiersState},
};

#[derive(Copy, Clone, Debug)]
pub struct InputSystem {
    is_left_clicked: bool,
    is_right_clicked: bool,
    is_control_w_clicked: bool,
    is_control_a_clicked: bool,
    is_control_s_clicked: bool,
    is_control_d_clicked: bool,
    cursor_delta: [f32; 2],
    wheel_delta: f32,
    modifiers: ModifiersState,
}

impl InputSystem {
    pub fn update(mut self, event: &Event<()>) -> Self {
        let mut is_left_clicked = None;
        let mut is_right_clicked = None;
        let mut wheel_delta = self.wheel_delta;
        let mut cursor_delta = self.cursor_delta;

        if let Event::NewEvents(_) = event {
            return Self {
                cursor_delta: [0.0, 0.0],
                wheel_delta: 0.0,
                ..self
            };
        }
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.modifiers = modifiers.state();
                    //println!("{:?}", self.modifiers);
                }
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic: false,
                    ..
                } => {
                    let mods = self.modifiers;

                    if event.state.is_pressed() {
                        let action = if let Key::Character(ch) = event.logical_key.as_ref() {
                            process_key_binding(&ch.to_uppercase(), &mods)
                        } else {
                            None
                        };

                        if let Some(action) = action {
                            self.handle_action(action, true);
                        }
                    } else {
                        let action = if let Key::Character(ch) = event.logical_key.as_ref() {
                            process_key_binding(&ch.to_uppercase(), &mods)
                        } else {
                            None
                        };

                        if let Some(action) = action {
                            self.handle_action(action, false);
                        }
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *state == ElementState::Pressed {
                        if *button == MouseButton::Left {
                            is_left_clicked = Some(true);
                        }
                        if *button == MouseButton::Right {
                            is_right_clicked = Some(true)
                        }
                    } else {
                        if *button == MouseButton::Left {
                            is_left_clicked = Some(false);
                        }
                        if *button == MouseButton::Right {
                            is_right_clicked = Some(false)
                        }
                    }
                }
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, v_lines),
                    ..
                } => {
                    wheel_delta += v_lines;
                }
                _ => {}
            }
        }

        if let Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta: (x, y) },
            ..
        } = event
        {
            cursor_delta[0] += *x as f32;
            cursor_delta[1] += *y as f32;
        }

        Self {
            is_left_clicked: is_left_clicked.unwrap_or(self.is_left_clicked),
            is_right_clicked: is_right_clicked.unwrap_or(self.is_right_clicked),
            is_control_w_clicked: self.is_control_w_clicked,
            is_control_a_clicked: self.is_control_a_clicked,
            is_control_s_clicked: self.is_control_s_clicked,
            is_control_d_clicked: self.is_control_d_clicked,
            cursor_delta,
            wheel_delta,
            modifiers: self.modifiers,
        }
    }

    fn handle_action(&mut self, action: Action, press: bool) {
        println!("{action:?} {press:?}");
        match action {
            Action::ControlW => self.is_control_w_clicked = press,
            Action::ControlA => self.is_control_a_clicked = press,
            Action::ControlS => self.is_control_s_clicked = press,
            Action::ControlD => self.is_control_d_clicked = press,
        }
    }
}

impl InputSystem {
    pub fn is_left_clicked(&self) -> bool {
        self.is_left_clicked
    }

    pub fn is_right_clicked(&self) -> bool {
        self.is_right_clicked
    }

    pub fn is_control_w_clicked(&self) -> bool {
        self.is_control_w_clicked
    }

    pub fn _is_control_a_clicked(&self) -> bool {
        self.is_control_a_clicked
    }

    pub fn is_control_s_clicked(&self) -> bool {
        self.is_control_s_clicked
    }

    pub fn _is_control_d_clicked(&self) -> bool {
        self.is_control_d_clicked
    }

    pub fn cursor_delta(&self) -> [f32; 2] {
        self.cursor_delta
    }

    pub fn wheel_delta(&self) -> f32 {
        self.wheel_delta
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self {
            is_left_clicked: false,
            is_right_clicked: false,
            is_control_w_clicked: false,
            is_control_a_clicked: false,
            is_control_s_clicked: false,
            is_control_d_clicked: false,
            cursor_delta: [0.0, 0.0],
            wheel_delta: 0.0,
            modifiers: Default::default(),
        }
    }
}

struct Binding<T: Eq> {
    trigger: T,
    mods: ModifiersState,
    action: Action,
}

impl<T: Eq> Binding<T> {
    const fn new(trigger: T, mods: ModifiersState, action: Action) -> Self {
        Self {
            trigger,
            mods,
            action,
        }
    }

    fn is_triggered_by(&self, trigger: &T, mods: &ModifiersState) -> bool {
        &self.trigger == trigger && &self.mods == mods
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    ControlW,
    ControlA,
    ControlS,
    ControlD,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

const KEY_BINDINGS: &[Binding<&'static str>] = &[
    Binding::new("W", ModifiersState::CONTROL, Action::ControlW),
    Binding::new("A", ModifiersState::CONTROL, Action::ControlA),
    Binding::new("S", ModifiersState::CONTROL, Action::ControlS),
    Binding::new("D", ModifiersState::CONTROL, Action::ControlD),
];

fn process_key_binding(key: &str, mods: &ModifiersState) -> Option<Action> {
    KEY_BINDINGS.iter().find_map(|binding| {
        binding
            .is_triggered_by(&key, mods)
            .then_some(binding.action)
    })
}

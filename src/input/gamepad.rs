// TODO: Remove this once WASM gamepad support is added
#![cfg_attr(target_arch = "wasm32", allow(unused))]

use hashbrown::{HashMap, HashSet};

use crate::math::Vec2;
use crate::platform;
use crate::Context;

pub(crate) struct GamepadState {
    pub platform_id: i32,
    pub current_button_state: HashSet<GamepadButton>,
    pub previous_button_state: HashSet<GamepadButton>,
    pub current_axis_state: HashMap<GamepadAxis, f32>,
}

impl GamepadState {
    pub(crate) fn new(platform_id: i32) -> GamepadState {
        GamepadState {
            platform_id,
            current_button_state: HashSet::new(),
            previous_button_state: HashSet::new(),
            current_axis_state: HashMap::new(),
        }
    }

    pub(crate) fn set_button_down(&mut self, btn: GamepadButton) {
        self.current_button_state.insert(btn);
    }

    pub(crate) fn set_button_up(&mut self, btn: GamepadButton) {
        self.current_button_state.remove(&btn);
    }

    pub(crate) fn set_axis_position(&mut self, axis: GamepadAxis, value: f32) {
        self.current_axis_state.insert(axis, value);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
/// A button on a gamepad.
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    Up,
    Down,
    Left,
    Right,
    LeftShoulder,
    LeftTrigger,
    LeftStick,
    RightShoulder,
    RightTrigger,
    RightStick,
    Start,
    Back,
    Guide,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
/// An axis of movement on a gamepad.
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftTrigger,
    RightStickX,
    RightStickY,
    RightTrigger,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
/// A control stick on a gamepad.
pub enum GamepadStick {
    LeftStick,
    RightStick,
}

/// Returns true if the specified gamepad is currently connected.
pub fn is_gamepad_connected(ctx: &Context, gamepad_index: usize) -> bool {
    get_gamepad(ctx, gamepad_index).is_some()
}

/// Returns the name of the specified gamepad, or `None` if it is not connected.
pub fn get_gamepad_name(ctx: &Context, gamepad_index: usize) -> Option<String> {
    get_gamepad(ctx, gamepad_index)
        .map(|g| g.platform_id)
        .map(|id| platform::get_gamepad_name(ctx, id))
}

/// Returns true if the specified gamepad button is currently down.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_down(ctx: &Context, gamepad_index: usize, button: GamepadButton) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        pad.current_button_state.contains(&button)
    } else {
        false
    }
}

/// Returns true if the specified gamepad button is currently up.
///
/// If the gamepad is disconnected, this will always return `true`.
pub fn is_gamepad_button_up(ctx: &Context, gamepad_index: usize, button: GamepadButton) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        !pad.current_button_state.contains(&button)
    } else {
        true
    }
}

/// Returns true if the specified gamepad button was pressed this tick.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_pressed(
    ctx: &Context,
    gamepad_index: usize,
    button: GamepadButton,
) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        !pad.previous_button_state.contains(&button) && pad.current_button_state.contains(&button)
    } else {
        false
    }
}

/// Returns true if the specified gamepad button was released this tick.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_released(
    ctx: &Context,
    gamepad_index: usize,
    button: GamepadButton,
) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        pad.previous_button_state.contains(&button) && !pad.current_button_state.contains(&button)
    } else {
        false
    }
}

enum GamepadIterator<T> {
    Disconnected,
    Connected(T),
}

impl<T> Iterator for GamepadIterator<T>
where
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        match self {
            GamepadIterator::Disconnected => None,
            GamepadIterator::Connected(i) => i.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GamepadIterator::Disconnected => (0, Some(0)),
            GamepadIterator::Connected(i) => i.size_hint(),
        }
    }
}

/// Returns an iterator of the buttons that are currently down on the specified gamepad.
///
/// If the gamepad is disconnected, an empty iterator will be returned.
pub fn get_gamepad_buttons_down(
    ctx: &Context,
    gamepad_index: usize,
) -> impl Iterator<Item = &GamepadButton> {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        GamepadIterator::Connected(pad.current_button_state.iter())
    } else {
        GamepadIterator::Disconnected
    }
}

/// Returns an iterator of the buttons that were pressed this tick on the specified gamepad.
///
/// If the gamepad is disconnected, an empty iterator will be returned.
pub fn get_gamepad_buttons_pressed(
    ctx: &Context,
    gamepad_index: usize,
) -> impl Iterator<Item = &GamepadButton> {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        GamepadIterator::Connected(
            pad.current_button_state
                .difference(&pad.previous_button_state),
        )
    } else {
        GamepadIterator::Disconnected
    }
}

/// Returns an iterator of the buttons that were released this tick on the specified gamepad.
///
/// If the gamepad is disconnected, an empty iterator will be returned.
pub fn get_gamepad_buttons_released(
    ctx: &Context,
    gamepad_index: usize,
) -> impl Iterator<Item = &GamepadButton> {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        GamepadIterator::Connected(
            pad.previous_button_state
                .difference(&pad.current_button_state),
        )
    } else {
        GamepadIterator::Disconnected
    }
}

/// Returns the current position of the specified gamepad axis.
///
/// If the gamepad is disconnected, this will always return `0.0`.
pub fn get_gamepad_axis_position(ctx: &Context, gamepad_index: usize, axis: GamepadAxis) -> f32 {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        if let Some(value) = pad.current_axis_state.get(&axis) {
            *value
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Returns the current position of the specified gamepad control stick.
///
/// If the gamepad is disconnected, this will always return `(0.0, 0.0)`.
pub fn get_gamepad_stick_position(
    ctx: &Context,
    gamepad_index: usize,
    stick: GamepadStick,
) -> Vec2<f32> {
    let (x_axis, y_axis) = match stick {
        GamepadStick::LeftStick => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
        GamepadStick::RightStick => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
    };

    Vec2::new(
        get_gamepad_axis_position(ctx, gamepad_index, x_axis),
        get_gamepad_axis_position(ctx, gamepad_index, y_axis),
    )
}

/// Returns whether or not the specified gamepad supports vibration.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_vibration_supported(ctx: &Context, gamepad_index: usize) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        platform::is_gamepad_vibration_supported(ctx, pad.platform_id)
    } else {
        false
    }
}

/// Sets the specified gamepad's motors to vibrate indefinitely.
pub fn set_gamepad_vibration(ctx: &mut Context, gamepad_index: usize, strength: f32) {
    if let Some(platform_id) = get_gamepad(ctx, gamepad_index).map(|g| g.platform_id) {
        platform::set_gamepad_vibration(ctx, platform_id, strength);
    }
}

/// Sets the specified gamepad's motors to vibrate for a set duration, specified in milliseconds.
/// After this time has passed, the vibration will automatically stop.
pub fn start_gamepad_vibration(
    ctx: &mut Context,
    gamepad_index: usize,
    strength: f32,
    duration: u32,
) {
    if let Some(platform_id) = get_gamepad(ctx, gamepad_index).map(|g| g.platform_id) {
        platform::start_gamepad_vibration(ctx, platform_id, strength, duration);
    }
}

/// Stops the specified gamepad's motors from vibrating.
pub fn stop_gamepad_vibration(ctx: &mut Context, gamepad_index: usize) {
    if let Some(platform_id) = get_gamepad(ctx, gamepad_index).map(|g| g.platform_id) {
        platform::stop_gamepad_vibration(ctx, platform_id);
    }
}

pub(crate) fn add_gamepad(ctx: &mut Context, platform_id: i32) -> usize {
    for (i, slot) in ctx.input.pads.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(GamepadState::new(platform_id));
            return i;
        }
    }

    // There wasn't an existing free slot...
    let i = ctx.input.pads.len();
    ctx.input.pads.push(Some(GamepadState::new(platform_id)));
    i
}

pub(crate) fn remove_gamepad(ctx: &mut Context, gamepad_index: usize) {
    ctx.input.pads[gamepad_index] = None;
}

pub(crate) fn get_gamepad(ctx: &Context, gamepad_index: usize) -> Option<&GamepadState> {
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
        Some(pad)
    } else {
        None
    }
}

pub(crate) fn get_gamepad_mut(
    ctx: &mut Context,
    gamepad_index: usize,
) -> Option<&mut GamepadState> {
    if let Some(Some(pad)) = ctx.input.pads.get_mut(gamepad_index) {
        Some(pad)
    } else {
        None
    }
}
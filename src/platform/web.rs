use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

use crate::audio::{RemoteControls, Sound, SoundInstance};
use crate::error::{Result, TetraError};
use crate::input::{self, Key};
use crate::{Context, Game, State};

pub type GlContext = glow::web::Context;

pub const DEFAULT_VERTEX_SHADER: &str = concat!(
    "#version 300 es\n",
    include_str!("../resources/shader.vert")
);

pub const DEFAULT_FRAGMENT_SHADER: &str = concat!(
    "#version 300 es\nprecision mediump float;\n",
    include_str!("../resources/shader.frag")
);

enum Event {
    KeyDown(Key),
    KeyUp(Key),
}

pub struct Platform {
    event_queue: Rc<RefCell<VecDeque<Event>>>,

    keydown_closure: Closure<dyn FnMut(KeyboardEvent)>,
    keyup_closure: Closure<dyn FnMut(KeyboardEvent)>,
}

impl Platform {
    pub fn new(builder: &Game) -> Result<(Platform, GlContext, i32, i32)> {
        // TODO: This is disgusting
        let document = web_sys::window()
            .ok_or_else(|| TetraError::Platform("Could not get 'window' from browser".into()))?
            .document()
            .ok_or_else(|| TetraError::Platform("Could not get 'document' from browser".into()))?;

        let context = document
            .get_element_by_id("canvas")
            .ok_or_else(|| TetraError::Platform("Could not find canvas element on page".into()))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| TetraError::Platform("Element was not a canvas".into()))?
            .get_context("webgl2")
            .map_err(|_| TetraError::Platform("Could not get context from canvas".into()))?
            .expect("webgl2 is a valid context type")
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();

        let event_queue = Rc::new(RefCell::new(VecDeque::new()));

        let event_queue_handle = Rc::clone(&event_queue);

        let keydown_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if let Some(key) = into_key(event) {
                event_queue_handle
                    .borrow_mut()
                    .push_back(Event::KeyDown(key));
            }
        }) as Box<dyn FnMut(_)>);

        document
            .add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
            .unwrap();

        let event_queue_handle = Rc::clone(&event_queue);

        let keyup_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if let Some(key) = into_key(event) {
                event_queue_handle.borrow_mut().push_back(Event::KeyUp(key));
            }
        }) as Box<dyn FnMut(_)>);

        document
            .add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())
            .unwrap();

        Ok((
            Platform {
                event_queue,

                keydown_closure,
                keyup_closure,
            },
            GlContext::from_webgl2_context(context),
            640,
            480,
        ))
    }
}

pub fn run_loop<S>(mut ctx: Context, mut state: S, frame: fn(&mut Context, &mut S))
where
    S: State + 'static,
{
    let callback = Rc::new(RefCell::new(None));
    let init = callback.clone();
    let refs = Rc::new(RefCell::new((ctx, state)));

    *init.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let (ctx, state) = &mut *refs.borrow_mut();
        frame(ctx, state);

        if ctx.running {
            request_animation_frame(callback.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));

    request_animation_frame(init.borrow().as_ref().unwrap());
}

pub fn handle_events(ctx: &mut Context) -> Result {
    while let Some(event) = {
        let mut x = ctx.platform.event_queue.borrow_mut();
        x.pop_front()
    } {
        match event {
            Event::KeyDown(key) => input::set_key_down(ctx, key),
            Event::KeyUp(key) => input::set_key_up(ctx, key),
        }
    }

    Ok(())
}

pub fn get_window_title(ctx: &Context) -> &str {
    ""
}

pub fn set_window_title<S>(ctx: &mut Context, title: S)
where
    S: AsRef<str>,
{

}

pub fn get_window_width(ctx: &Context) -> i32 {
    640
}

pub fn get_window_height(ctx: &Context) -> i32 {
    480
}

pub fn get_window_size(ctx: &Context) -> (i32, i32) {
    (640, 480)
}

pub fn set_window_size(ctx: &mut Context, width: i32, height: i32) {}

pub fn toggle_fullscreen(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn enable_fullscreen(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn disable_fullscreen(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn is_fullscreen(ctx: &Context) -> bool {
    false
}

pub fn set_mouse_visible(ctx: &mut Context, mouse_visible: bool) {}

pub fn is_mouse_visible(ctx: &Context) -> bool {
    true
}

pub fn swap_buffers(ctx: &Context) {}

pub fn get_gamepad_name(ctx: &Context, platform_id: i32) -> String {
    String::new()
}

pub fn is_gamepad_vibration_supported(ctx: &Context, platform_id: i32) -> bool {
    false
}

pub fn set_gamepad_vibration(ctx: &mut Context, platform_id: i32, strength: f32) {}

pub fn start_gamepad_vibration(ctx: &mut Context, platform_id: i32, strength: f32, duration: u32) {}

pub fn stop_gamepad_vibration(ctx: &mut Context, platform_id: i32) {}

// TODO: Find a better way of stubbing the audio stuff out.

pub fn play_sound(
    ctx: &Context,
    sound: &Sound,
    playing: bool,
    repeating: bool,
    volume: f32,
    speed: f32,
) -> Result<SoundInstance> {
    let controls = Arc::new(RemoteControls {
        playing: AtomicBool::new(playing),
        repeating: AtomicBool::new(repeating),
        rewind: AtomicBool::new(false),
        volume: Mutex::new(volume),
        speed: Mutex::new(speed),
    });

    Ok(SoundInstance { controls })
}

pub fn set_master_volume(ctx: &mut Context, volume: f32) {}

pub fn get_master_volume(ctx: &mut Context) -> f32 {
    1.0
}

fn into_key(event: KeyboardEvent) -> Option<Key> {
    let location = event.location();

    match (event.key().as_ref(), event.location()) {
        ("a", _) | ("A", _) => Some(Key::A),
        ("b", _) | ("B", _) => Some(Key::B),
        ("c", _) | ("C", _) => Some(Key::C),
        ("d", _) | ("D", _) => Some(Key::D),
        ("e", _) | ("E", _) => Some(Key::E),
        ("f", _) | ("F", _) => Some(Key::F),
        ("g", _) | ("G", _) => Some(Key::G),
        ("h", _) | ("H", _) => Some(Key::H),
        ("i", _) | ("I", _) => Some(Key::I),
        ("j", _) | ("J", _) => Some(Key::J),
        ("k", _) | ("K", _) => Some(Key::K),
        ("l", _) | ("L", _) => Some(Key::L),
        ("m", _) | ("M", _) => Some(Key::M),
        ("n", _) | ("N", _) => Some(Key::N),
        ("o", _) | ("O", _) => Some(Key::O),
        ("p", _) | ("P", _) => Some(Key::P),
        ("q", _) | ("Q", _) => Some(Key::Q),
        ("r", _) | ("R", _) => Some(Key::R),
        ("s", _) | ("S", _) => Some(Key::S),
        ("t", _) | ("T", _) => Some(Key::T),
        ("u", _) | ("U", _) => Some(Key::U),
        ("v", _) | ("V", _) => Some(Key::V),
        ("w", _) | ("W", _) => Some(Key::W),
        ("x", _) | ("X", _) => Some(Key::X),
        ("y", _) | ("Y", _) => Some(Key::Y),
        ("z", _) | ("Z", _) => Some(Key::Z),

        ("0", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num0),
        ("1", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num1),
        ("2", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num2),
        ("3", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num3),
        ("4", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num4),
        ("5", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num5),
        ("6", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num6),
        ("7", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num7),
        ("8", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num8),
        ("9", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num9),

        ("F1", _) => Some(Key::F1),
        ("F2", _) => Some(Key::F2),
        ("F3", _) => Some(Key::F3),
        ("F4", _) => Some(Key::F4),
        ("F5", _) => Some(Key::F5),
        ("F6", _) => Some(Key::F6),
        ("F7", _) => Some(Key::F7),
        ("F8", _) => Some(Key::F8),
        ("F9", _) => Some(Key::F9),
        ("F10", _) => Some(Key::F10),
        ("F11", _) => Some(Key::F11),
        ("F12", _) => Some(Key::F12),
        ("F13", _) => Some(Key::F13),
        ("F14", _) => Some(Key::F14),
        ("F15", _) => Some(Key::F15),
        ("F16", _) => Some(Key::F16),
        ("F17", _) => Some(Key::F17),
        ("F18", _) => Some(Key::F18),
        ("F19", _) => Some(Key::F19),
        ("F20", _) => Some(Key::F20),
        ("F21", _) => Some(Key::F21),
        ("F22", _) => Some(Key::F22),
        ("F23", _) => Some(Key::F23),
        ("F24", _) => Some(Key::F24),

        ("NumLock", _) => Some(Key::NumLock),
        ("0", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad1),
        ("1", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad2),
        ("2", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad3),
        ("3", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad4),
        ("4", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad5),
        ("5", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad6),
        ("6", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad7),
        ("7", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad8),
        ("8", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad9),
        ("9", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad0),
        ("+", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadPlus),
        ("-", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadMinus),
        ("*", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadMultiply),
        ("/", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadDivide),
        ("Enter", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadEnter),

        ("Control", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::LeftCtrl),
        ("Shift", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::LeftShift),
        ("Alt", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::LeftAlt),
        ("Control", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::RightCtrl),
        ("Shift", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::RightShift),
        ("Alt", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::RightAlt),

        ("ArrowUp", _) => Some(Key::Up),
        ("ArrowDown", _) => Some(Key::Down),
        ("ArrowLeft", _) => Some(Key::Left),
        ("ArrowRight", _) => Some(Key::Right),

        ("&", _) => Some(Key::Ampersand),
        ("*", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Asterisk),
        ("@", _) => Some(Key::At),
        ("`", _) => Some(Key::Backquote),
        ("\\", _) => Some(Key::Backslash),
        ("Backspace", _) => Some(Key::Backspace),
        ("CapsLock", _) => Some(Key::CapsLock),
        ("^", _) => Some(Key::Caret),
        (":", _) => Some(Key::Colon),
        (",", _) => Some(Key::Comma),
        ("Delete", _) => Some(Key::Delete),
        ("$", _) => Some(Key::Dollar),
        ("\"", _) => Some(Key::DoubleQuote),
        ("End", _) => Some(Key::End),
        ("Enter", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Enter),
        ("=", _) => Some(Key::Equals),
        ("Escape", _) => Some(Key::Escape),
        ("!", _) => Some(Key::Exclaim),
        (">", _) => Some(Key::GreaterThan),
        ("#", _) => Some(Key::Hash),
        ("Home", _) => Some(Key::Home),
        ("Insert", _) => Some(Key::Insert),
        ("{", _) => Some(Key::LeftBracket),
        ("(", _) => Some(Key::LeftParen),
        ("<", _) => Some(Key::LessThan),
        ("-", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Minus),
        ("PageDown", _) => Some(Key::PageDown),
        ("PageUp", _) => Some(Key::PageUp),
        ("Pause", _) => Some(Key::Pause),
        ("%", _) => Some(Key::Percent),
        (".", _) => Some(Key::Period),
        ("+", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Plus),
        ("PrintScreen", _) => Some(Key::PrintScreen),
        ("?", _) => Some(Key::Question),
        ("'", _) => Some(Key::Quote),
        ("}", _) => Some(Key::RightBracket),
        (")", _) => Some(Key::RightParen),
        ("ScrollLock", _) => Some(Key::ScrollLock),
        (";", _) => Some(Key::Semicolon),
        ("/", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Slash),
        (" ", _) => Some(Key::Space),
        ("Tab", _) => Some(Key::Tab),
        ("_", _) => Some(Key::Underscore),

        _ => None,
    }
}

#[derive(Debug)]
pub struct DecoderError;

impl Display for DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "dummy decoder error")
    }
}

impl Error for DecoderError {}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
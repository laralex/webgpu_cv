//! This crate provides a backend platform for imgui-rs
//! based on JavaScript events accessed through web_sys crate.
//!
//! A backend platform handles window/input device events and manages their
//! state.
//! 
//! The source code is adapted from imgui-rs/imgui-winit-support
//!
//! # Using the library
//!
//! There are five things you need to do to use this library correctly:
//!
//! 1. Initialize a `JsPlatform` instance
//! N. Call frame preparation callback (every frame)
//! N+1. Call render preparation callback (every frame)
//!
//! ## Complete example (without a renderer)
//! ```no_run
//! ```

use imgui::{self, BackendFlags, ConfigFlags, Context, Io, Key, Ui};
use std::{cmp::Ordering, collections::HashMap, hash::RandomState};

// Re-export winit to make it easier for users to use the correct version.
use web_sys::MouseEvent;

/// winit backend platform state
#[derive(Debug)]
pub struct WebsysPlatform {
    hidpi_mode: ActiveHiDpiMode,
    hidpi_factor: f64,
    cursor_cache: Option<CursorSettings>,
    keycode_to_imgui: HashMap<&'static str, imgui::Key>,
}

fn to_css_cursor<'a>(cursor: imgui::MouseCursor) -> &'a str {
    match cursor {
        imgui::MouseCursor::Arrow => "default",
        imgui::MouseCursor::TextInput => "text",
        imgui::MouseCursor::ResizeAll => "move",
        imgui::MouseCursor::ResizeNS => "ns-resize",
        imgui::MouseCursor::ResizeEW => "ew-resize",
        imgui::MouseCursor::ResizeNESW => "nesw-resize",
        imgui::MouseCursor::ResizeNWSE => "nwse-resize",
        imgui::MouseCursor::Hand => "pointer",
        imgui::MouseCursor::NotAllowed => "none",
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct CursorSettings {
    cursor: Option<imgui::MouseCursor>,
    draw_cursor: bool,
}

impl CursorSettings {
    fn apply(&self, element: &web_sys::CssRule) {
        match self.cursor {
            Some(mouse_cursor) if !self.draw_cursor => {
                element.set_css_text(to_css_cursor(mouse_cursor));
            },
            _ => {},
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ActiveHiDpiMode {
    Default,
    Rounded,
    Locked,
}

/// DPI factor handling mode.
///
/// Applications that use imgui-rs might want to customize the used DPI factor and not use
/// directly the value coming from winit.
///
/// **Note: if you use a mode other than default and the DPI factor is adjusted, winit and imgui-rs
/// will use different logical coordinates, so be careful if you pass around logical size or
/// position values.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HiDpiMode {
    /// The DPI factor from winit is used directly without adjustment
    Default,
    /// The DPI factor from winit is rounded to an integer value.
    ///
    /// This prevents the user interface from becoming blurry with non-integer scaling.
    Rounded,
    /// The DPI factor from winit is ignored, and the included value is used instead.
    ///
    /// This is useful if you want to force some DPI factor (e.g. 1.0) and not care about the value
    /// coming from winit.
    Locked(f64),
}

impl HiDpiMode {
    fn apply(&self, hidpi_factor: f64) -> (ActiveHiDpiMode, f64) {
        match *self {
            HiDpiMode::Default => (ActiveHiDpiMode::Default, hidpi_factor),
            HiDpiMode::Rounded => (ActiveHiDpiMode::Rounded, hidpi_factor.round()),
            HiDpiMode::Locked(value) => (ActiveHiDpiMode::Locked, value),
        }
    }
}

// Logical position on a screen in floats. Adopted from winit-rs
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct LogicalPosition {
    pub x: f64,
    pub y: f64,
}

impl LogicalPosition {
    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        LogicalPosition { x, y }
    }
}

impl LogicalPosition {
    #[inline]
    pub fn from_physical<T: Into<PhysicalPosition>>(physical: T, scale_factor: f64) -> Self {
        physical.into().to_logical(scale_factor)
    }

    #[inline]
    pub fn to_physical(&self, scale_factor: f64) -> PhysicalPosition {
        let x = (self.x * scale_factor).round() as usize;
        let y = (self.y * scale_factor).round() as usize;
        PhysicalPosition::new(x, y)
    }
}

/// A position represented in physical pixels. Adopted from winit-rs
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct PhysicalPosition {
    pub x: usize,
    pub y: usize,
}

impl PhysicalPosition {
    #[inline]
    pub const fn new(x: usize, y: usize) -> Self {
        PhysicalPosition { x, y }
    }

    #[inline]
    pub fn from_logical<T: Into<LogicalPosition>>(logical: T, scale_factor: f64) -> Self {
        logical.into().to_physical(scale_factor)
    }

    #[inline]
    pub fn to_logical(&self, scale_factor: f64) -> LogicalPosition {
        let x = self.x as f64 / scale_factor;
        let y = self.y as f64 / scale_factor;
        LogicalPosition::new(x, y)
    }
}

/// A size represented in logical pixels. Adopted from winit-rs
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct LogicalSize {
    pub width: f64,
    pub height: f64,
}

impl LogicalSize {
    #[inline]
    pub const fn new(width: f64, height: f64) -> Self {
        LogicalSize { width, height }
    }
}

impl LogicalSize {
    #[inline]
    pub fn from_physical<T: Into<PhysicalSize>>(physical: T, scale_factor: f64) -> Self {
        physical.into().to_logical(scale_factor)
    }

    #[inline]
    pub fn to_physical(&self, scale_factor: f64) -> PhysicalSize {
        let width = (self.width * scale_factor).round() as usize;
        let height = (self.height * scale_factor).round() as usize;
        PhysicalSize::new(width, height)
    }
}

/// A size represented in physical pixels. Adopted from winit-rs
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct PhysicalSize {
    pub width: usize,
    pub height: usize,
}

impl PhysicalSize {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        PhysicalSize { width, height }
    }
}

impl PhysicalSize {
    #[inline]
    pub fn from_logical<T: Into<LogicalSize>>(logical: T, scale_factor: f64) -> Self {
        logical.into().to_physical(scale_factor)
    }

    #[inline]
    pub fn to_logical(&self, scale_factor: f64) -> LogicalSize {
        let width = self.width as f64 / scale_factor;
        let height = self.height as f64 / scale_factor;
        LogicalSize::new(width, height)
    }
}

fn to_imgui_mouse_button(event: &web_sys::MouseEvent) -> Option<imgui::MouseButton> {
    match event.button() {
        0 => Some(imgui::MouseButton::Left),
        1 => Some(imgui::MouseButton::Right),
        2 => Some(imgui::MouseButton::Middle),
        _ => None,
    }
}

fn make_keycode_map() -> HashMap<&'static str, imgui::Key> {
    const KEY_CAPACITY: usize = 200;
    let hasher = RandomState::new();
    let mut keycode_map = HashMap::with_capacity_and_hasher(KEY_CAPACITY, hasher);
    keycode_map.insert("Tab", Key::Tab);
    keycode_map.insert("ArrowLeft", Key::LeftArrow);
    keycode_map.insert("ArrowRight", Key::RightArrow);
    keycode_map.insert("ArrowUp", Key::UpArrow);
    keycode_map.insert("ArrowDown", Key::DownArrow);
    keycode_map.insert("PageUp", Key::PageUp);
    keycode_map.insert("PageDown", Key::PageDown);
    keycode_map.insert("Home", Key::Home);
    keycode_map.insert("End", Key::End);
    keycode_map.insert("Insert", Key::Insert);
    keycode_map.insert("Delete", Key::Delete);
    keycode_map.insert("Backspace", Key::Backspace);
    keycode_map.insert("Enter", Key::Enter);
    keycode_map.insert("NumpadEnter", Key::KeypadEnter);
    keycode_map.insert("Escape", Key::Escape);
    keycode_map.insert("ControlLeft", Key::LeftCtrl);
    keycode_map.insert("ControlRight", Key::RightCtrl);
    keycode_map.insert("ShiftLeft", Key::LeftShift);
    keycode_map.insert("ShiftRight", Key::RightShift);
    keycode_map.insert("AltLeft", Key::LeftAlt);
    keycode_map.insert("AltRight", Key::RightAlt);
    keycode_map.insert("MetaLeft", Key::LeftSuper);
    keycode_map.insert("MetaRight", Key::RightSuper);
    keycode_map.insert("ContextMenu", Key::Menu);
    keycode_map.insert("F1", Key::F1);
    keycode_map.insert("F2", Key::F2);
    keycode_map.insert("F3", Key::F3);
    keycode_map.insert("F4", Key::F4);
    keycode_map.insert("F5", Key::F5);
    keycode_map.insert("F6", Key::F6);
    keycode_map.insert("F7", Key::F7);
    keycode_map.insert("F8", Key::F8);
    keycode_map.insert("F9", Key::F9);
    keycode_map.insert("F10", Key::F10);
    keycode_map.insert("F11", Key::F11);
    keycode_map.insert("F12", Key::F12);
    keycode_map.insert("CapsLock", Key::CapsLock);
    keycode_map.insert("ScrollLock", Key::ScrollLock);
    keycode_map.insert("NumLock", Key::NumLock);
    keycode_map.insert("PrintScreen", Key::PrintScreen);
    keycode_map.insert("Pause", Key::Pause);
    keycode_map.insert("Digit0", Key::Alpha0);
    keycode_map.insert("Digit1", Key::Alpha1);
    keycode_map.insert("Digit2", Key::Alpha2);
    keycode_map.insert("Digit3", Key::Alpha3);
    keycode_map.insert("Digit4", Key::Alpha4);
    keycode_map.insert("Digit5", Key::Alpha5);
    keycode_map.insert("Digit6", Key::Alpha6);
    keycode_map.insert("Digit7", Key::Alpha7);
    keycode_map.insert("Digit8", Key::Alpha8);
    keycode_map.insert("Digit9", Key::Alpha9);
    keycode_map.insert("Numpad0", Key::Keypad0);
    keycode_map.insert("Numpad1", Key::Keypad1);
    keycode_map.insert("Numpad2", Key::Keypad2);
    keycode_map.insert("Numpad3", Key::Keypad3);
    keycode_map.insert("Numpad4", Key::Keypad4);
    keycode_map.insert("Numpad5", Key::Keypad5);
    keycode_map.insert("Numpad6", Key::Keypad6);
    keycode_map.insert("Numpad7", Key::Keypad7);
    keycode_map.insert("Numpad8", Key::Keypad8);
    keycode_map.insert("Numpad9", Key::Keypad9);
    keycode_map.insert("KeyA", Key::A);
    keycode_map.insert("KeyB", Key::B);
    keycode_map.insert("KeyC", Key::C);
    keycode_map.insert("KeyD", Key::D);
    keycode_map.insert("KeyE", Key::E);
    keycode_map.insert("KeyF", Key::F);
    keycode_map.insert("KeyG", Key::G);
    keycode_map.insert("KeyH", Key::H);
    keycode_map.insert("KeyI", Key::I);
    keycode_map.insert("KeyJ", Key::J);
    keycode_map.insert("KeyK", Key::K);
    keycode_map.insert("KeyL", Key::L);
    keycode_map.insert("KeyM", Key::M);
    keycode_map.insert("KeyN", Key::N);
    keycode_map.insert("KeyO", Key::O);
    keycode_map.insert("KeyP", Key::P);
    keycode_map.insert("KeyQ", Key::Q);
    keycode_map.insert("KeyR", Key::R);
    keycode_map.insert("KeyS", Key::S);
    keycode_map.insert("KeyT", Key::T);
    keycode_map.insert("KeyU", Key::U);
    keycode_map.insert("KeyV", Key::V);
    keycode_map.insert("KeyW", Key::W);
    keycode_map.insert("KeyX", Key::X);
    keycode_map.insert("KeyY", Key::Y);
    keycode_map.insert("KeyZ", Key::Z);
    keycode_map.insert("Quote", Key::Apostrophe);
    keycode_map.insert("Comma", Key::Comma);
    keycode_map.insert("Minus", Key::Minus);
    keycode_map.insert("NumpadSubtract", Key::KeypadSubtract);
    keycode_map.insert("Period", Key::Period);
    keycode_map.insert("NumpadDecimal", Key::KeypadDecimal);
    keycode_map.insert("Slash", Key::Slash);
    keycode_map.insert("NumpadDivide", Key::KeypadDivide);
    keycode_map.insert("Semicolon", Key::Semicolon);
    keycode_map.insert("Equal", Key::Equal);
    keycode_map.insert("NumpadEqual", Key::KeypadEqual);
    keycode_map.insert("BracketLeft", Key::LeftBracket);
    keycode_map.insert("BracketRight", Key::RightBracket);
    keycode_map.insert("Backslash", Key::Backslash);
    keycode_map.insert("Backquote", Key::GraveAccent);
    keycode_map.insert("NumpadMultiply", Key::KeypadMultiply);
    keycode_map.insert("NumpadAdd", Key::KeypadAdd);
    keycode_map
}

pub struct MouseEvent{
    pub event: web_sys::MouseEvent,
    pub is_down: bool,
}
pub struct KeyboardEvent{
    pub event: web_sys::KeyboardEvent,
    pub is_down: bool,
}
pub struct MouseWheelEvent {
    pub norm_delta_x: f32,
    pub norm_delta_y: f32,
}
pub enum WebEvent {
    Resized(PhysicalSize),
    HiDpiFactorChanged(PhysicalSize),
    Keyboard(KeyboardEvent),
    CursorMoved(PhysicalPosition),
    Mouse(MouseEvent),
    MouseWheel(MouseWheelEvent),
    Focused(bool),
};

impl WebsysPlatform {
    /// Initializes a winit platform instance and configures imgui.
    ///
    /// This function configures imgui-rs in the following ways:
    ///
    /// * backend flags are updated
    /// * keys are configured
    /// * platform name is set
    pub fn init(imgui: &mut Context, display_size: PhysicalSize, display_hidpi_scale: f64, hidpi_mode: HiDpiMode) -> WebsysPlatform {
        let io = imgui.io_mut();

        // NOTE: not supported
        // io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
        // io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
        imgui.set_platform_name(Some(format!(
            "imgui-web-sys-support {}",
            "0.0.1"
        )));

        let mut platform = WebsysPlatform {
            hidpi_mode: ActiveHiDpiMode::Default,
            hidpi_factor: 1.0,
            cursor_cache: None,
            keycode_to_imgui: make_keycode_map(),
        };
        platform.update_display_size(io, display_size, display_hidpi_scale, hidpi_mode);
        platform
    }

    fn to_imgui_key(&self, event: &web_sys::KeyboardEvent) -> Option<imgui::Key> {
        self.keycode_to_imgui.get(event.code.into::<&str>()).copied()
    }

    /// Attaches the platform instance to a winit window.
    ///
    /// This function configures imgui-rs in the following ways:
    ///
    /// * framebuffer scale (= DPI factor) is set
    /// * display size is set
    fn update_display_size(&mut self, io: &mut Io, display_size: PhysicalSize, display_hidpi_scale: f64, hidpi_mode: HiDpiMode ) {
        let (hidpi_mode, hidpi_factor) = hidpi_mode.apply(display_hidpi_scale);
        self.hidpi_mode = hidpi_mode;
        self.hidpi_factor = hidpi_factor;
        io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
        let logical_size = display_size.to_logical(hidpi_factor);
        let logical_size = self.scale_size_from_external_factor(display_hidpi_scale, logical_size);
        io.display_size = [logical_size.width as f32, logical_size.height as f32];
    }
    /// Returns the current DPI factor.
    ///
    /// The value might not be the same as the winit DPI factor (depends on the used DPI mode)
    pub fn hidpi_factor(&self) -> f64 {
        self.hidpi_factor
    }
    /// Scales a logical size coming from external module using the current DPI mode.
    ///
    /// This utility function is useful if you are using a DPI mode other than default, and want
    /// your application to use the same logical coordinates as imgui-rs.
    pub fn scale_size_from_external_factor(
        &self,
        external_hidpi_factor: f64,
        logical_size: LogicalSize,
    ) -> LogicalSize {
        match self.hidpi_mode {
            ActiveHiDpiMode::Default => logical_size,
            _ => logical_size
                .to_physical(external_hidpi_factor)
                .to_logical(self.hidpi_factor),
        }
    }
    /// Scales a logical position coming from external module using the current DPI mode.
    ///
    /// This utility function is useful if you are using a DPI mode other than default, and want
    /// your application to use the same logical coordinates as imgui-rs.
    pub fn scale_pos_from_external_factor(
        &self,
        external_hidpi_factor: f64,
        logical_pos: LogicalPosition,
    ) -> LogicalPosition {
        match self.hidpi_mode {
            ActiveHiDpiMode::Default => logical_pos,
            _ => logical_pos
                .to_physical(external_hidpi_factor)
                .to_logical(self.hidpi_factor),
        }
    }
    /// Scales a logical position for external module using the current DPI mode.
    ///
    /// This utility function is useful if you are using a DPI mode other than default, and want
    /// your application to use the same logical coordinates as imgui-rs.
    pub fn scale_pos_for_external_factor(
        &self,
        external_hidpi_factor: f64,
        logical_pos: LogicalPosition,
    ) -> LogicalPosition {
        match self.hidpi_mode {
            ActiveHiDpiMode::Default => logical_pos,
            _ => logical_pos
                .to_physical(self.hidpi_factor)
                .to_logical(external_hidpi_factor),
        }
    }
    /// Handles a winit event.
    ///
    /// This function performs the following actions (depends on the event):
    ///
    /// * window size / dpi factor changes are applied
    /// * keyboard state is updated
    /// * mouse state is updated
    pub fn handle_event(&mut self, io: &mut Io, external_hidpi_factor: f64, event: &WebEvent) {
        match *event {
            WebEvent::Resized(physical_size) => {
                let logical_size = physical_size.to_logical(external_hidpi_factor);
                let logical_size = self.scale_size_from_external_factor(external_hidpi_factor, logical_size);
                io.display_size = [logical_size.width as f32, logical_size.height as f32];
            },
            WebEvent::HiDpiFactorChanged(physical_size) => {
                let hidpi_factor = match self.hidpi_mode {
                    ActiveHiDpiMode::Default => external_hidpi_factor,
                    ActiveHiDpiMode::Rounded => external_hidpi_factor.round(),
                    _ => return,
                };
                // Mouse position needs to be changed while we still have both the old and the new
                // values
                if io.mouse_pos[0].is_finite() && io.mouse_pos[1].is_finite() {
                    io.mouse_pos = [
                        io.mouse_pos[0] * (hidpi_factor / self.hidpi_factor) as f32,
                        io.mouse_pos[1] * (hidpi_factor / self.hidpi_factor) as f32,
                    ];
                }
                self.hidpi_factor = hidpi_factor;
                io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
                // Window size might change too if we are using DPI rounding
                let logical_size = physical_size.to_logical(external_hidpi_factor);
                let logical_size = self.scale_size_from_external_factor(external_hidpi_factor, logical_size);
                io.display_size = [logical_size.width as f32, logical_size.height as f32];
            },
            WebEvent::Keyboard(KeyboardEvent{event, is_down}) => {
                // NOTE: text input is not yet handled
                // if let Some(txt) = &event.text {
                //     for ch in txt.chars() {
                //         if ch != '\u{7f}' {
                //             io.add_input_character(ch)
                //         }
                //     }
                // }

                // We map both left and right ctrl to `ModCtrl`, etc.
                // imgui is told both "left control is pressed" and
                // "consider the control key is pressed". Allows
                // applications to use either general "ctrl" or a
                // specific key. Same applies to other modifiers.
                // https://github.com/ocornut/imgui/issues/5047
                io.add_key_event(imgui::Key::ModShift, event.shift_key());
                io.add_key_event(imgui::Key::ModCtrl, event.ctrl_key());
                io.add_key_event(imgui::Key::ModAlt, event.alt_key());
                io.add_key_event(imgui::Key::ModSuper, event.meta_key());

                // Add main key event
                if let Some(key) = self.to_imgui_key(&event) {
                    io.add_key_event(key, is_down);
                }
            },
            WebEvent::CursorMoved(physical_position) => {
                let position = physical_position.to_logical(external_hidpi_factor);
                let position = self.scale_pos_from_external_factor(external_hidpi_factor, position);
                io.add_mouse_pos_event([position.x as f32, position.y as f32]);
            },
            WebEvent::Mouse(MouseEvent{event, is_down}) => {
                if let Some(mb) = to_imgui_mouse_button(&event) {
                    io.add_mouse_button_event(mb, is_down);
                }
            },
            WebEvent::MouseWheel(MouseWheelEvent{norm_delta_x, norm_delta_y}) => {
                io.add_mouse_wheel_event(
                    [norm_delta_x, norm_delta_y]);
            },
            WebEvent::Focused(is_focused) => {
                if !is_focused {
                    // Set focus-lost to avoid stuck keys (like 'alt'
                    // when alt-tabbing)
                    io.app_focus_lost = true;
                }
            },
            _ => (),
        }
    }

    /// Frame preparation callback.
    ///
    /// Call this before calling the imgui-rs context `frame` function.
    /// This function performs the following actions:
    ///
    /// * mouse cursor is repositioned (if requested by imgui-rs)
    pub fn prepare_frame(&self, io: &mut Io) {
        if io.want_set_mouse_pos {
            let _x = f64::from(io.mouse_pos[0]);
            let _y = f64::from(io.mouse_pos[1]);
            // NOTE: not supported by browsers
            // NOTE: not enabled on imgui: io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
            // window.set_cursor_position(logical_pos)
        }
    }

    /// Render preparation callback.
    ///
    /// Call this before calling the imgui-rs UI `render_with`/`render` function.
    /// This function performs the following actions:
    ///
    /// * mouse cursor is changed and/or hidden (if requested by imgui-rs)
    pub fn prepare_render(&mut self, ui: &Ui) {
        let io = ui.io();
        if !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
        {
            let cursor = CursorSettings {
                cursor: ui.mouse_cursor(),
                draw_cursor: io.mouse_draw_cursor,
            };
            if self.cursor_cache != Some(cursor) {
                // NOTE: no cursor handling for now
                // NOTE: not enabled on imgui: io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS)
                // cursor.apply(window);
                self.cursor_cache = Some(cursor);
            }
        }
    }
}

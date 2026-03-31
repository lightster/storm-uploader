use crate::input_recorder::RawInputEvent;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::SystemTime;

// CGEventType constants (from CoreGraphics)
const CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
const CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
const CG_EVENT_RIGHT_MOUSE_UP: u32 = 4;
const CG_EVENT_KEY_DOWN: u32 = 10;
const CG_EVENT_KEY_UP: u32 = 11;
const CG_EVENT_FLAGS_CHANGED: u32 = 12;
const CG_EVENT_OTHER_MOUSE_DOWN: u32 = 25;
const CG_EVENT_OTHER_MOUSE_UP: u32 = 26;

// CGEventField values
const KEYBOARD_EVENT_KEYCODE: u32 = 9; // kCGKeyboardEventKeycode
const MOUSE_EVENT_BUTTON_NUMBER: u32 = 3; // kCGMouseEventButtonNumber

// CGEventTapLocation / placement / options
const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

// Event mask: keyboard + mouse buttons only (no mouse move, scroll, drag)
const EVENT_MASK: u64 = (1 << CG_EVENT_LEFT_MOUSE_DOWN)
    | (1 << CG_EVENT_LEFT_MOUSE_UP)
    | (1 << CG_EVENT_RIGHT_MOUSE_DOWN)
    | (1 << CG_EVENT_RIGHT_MOUSE_UP)
    | (1 << CG_EVENT_KEY_DOWN)
    | (1 << CG_EVENT_KEY_UP)
    | (1 << CG_EVENT_FLAGS_CHANGED)
    | (1 << CG_EVENT_OTHER_MOUSE_DOWN)
    | (1 << CG_EVENT_OTHER_MOUSE_UP);

type EventTapCallback = unsafe extern "C" fn(
    proxy: *mut c_void,
    event_type: u32,
    cg_event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: EventTapCallback,
        user_info: *mut c_void,
    ) -> *mut c_void;
    fn CGEventTapEnable(tap: *mut c_void, enable: bool);
    fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
    fn CGEventGetFlags(event: *mut c_void) -> u64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopStop(rl: *mut c_void);
    fn CFRunLoopRun();
    fn CFRetain(cf: *const c_void) -> *const c_void;
    fn CFRelease(cf: *const c_void);
    static kCFRunLoopCommonModes: *const c_void;
}

extern "C" {
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

struct ListenerState {
    tx: Sender<RawInputEvent>,
    recording: Arc<AtomicBool>,
    last_flags: u64,
}

/// Thread-safe wrapper around a CFRunLoopRef for stopping the listener from another thread.
pub struct RunLoopRef(*mut c_void);
unsafe impl Send for RunLoopRef {}

impl RunLoopRef {
    pub fn stop(&self) {
        if !self.0.is_null() {
            unsafe { CFRunLoopStop(self.0) }
        }
    }
}

impl Drop for RunLoopRef {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as *const c_void) }
        }
    }
}

unsafe extern "C" fn raw_callback(
    _proxy: *mut c_void,
    event_type: u32,
    cg_event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void {
    let state = &mut *(user_info as *mut ListenerState);

    if !state.recording.load(Ordering::Relaxed) {
        return cg_event;
    }

    if let Some(raw_event) = convert_event(event_type, cg_event, &mut state.last_flags) {
        let _ = state.tx.send(raw_event);
    }

    cg_event
}

unsafe fn convert_event(
    event_type: u32,
    cg_event: *mut c_void,
    last_flags: &mut u64,
) -> Option<RawInputEvent> {
    let (type_str, key, raw) = match event_type {
        CG_EVENT_KEY_DOWN => {
            let code = CGEventGetIntegerValueField(cg_event, KEYBOARD_EVENT_KEYCODE) as u16;
            let name = key_name_from_code(code);
            let raw = if name.starts_with("Unknown(") { code as u32 } else { 0 };
            ("kd", name, raw)
        }
        CG_EVENT_KEY_UP => {
            let code = CGEventGetIntegerValueField(cg_event, KEYBOARD_EVENT_KEYCODE) as u16;
            let name = key_name_from_code(code);
            let raw = if name.starts_with("Unknown(") { code as u32 } else { 0 };
            ("ku", name, raw)
        }
        CG_EVENT_FLAGS_CHANGED => {
            let code = CGEventGetIntegerValueField(cg_event, KEYBOARD_EVENT_KEYCODE) as u16;
            let flags = CGEventGetFlags(cg_event);
            let type_str: &'static str = if flags < *last_flags { "ku" } else { "kd" };
            *last_flags = flags;
            let name = key_name_from_code(code);
            let raw = if name.starts_with("Unknown(") { code as u32 } else { 0 };
            (type_str, name, raw)
        }
        CG_EVENT_LEFT_MOUSE_DOWN => ("bd", "Left".to_string(), 1),
        CG_EVENT_LEFT_MOUSE_UP => ("bu", "Left".to_string(), 1),
        CG_EVENT_RIGHT_MOUSE_DOWN => ("bd", "Right".to_string(), 2),
        CG_EVENT_RIGHT_MOUSE_UP => ("bu", "Right".to_string(), 2),
        CG_EVENT_OTHER_MOUSE_DOWN => {
            let btn = CGEventGetIntegerValueField(cg_event, MOUSE_EVENT_BUTTON_NUMBER) as u32;
            let (name, raw) = other_mouse_button_name(btn);
            ("bd", name, raw)
        }
        CG_EVENT_OTHER_MOUSE_UP => {
            let btn = CGEventGetIntegerValueField(cg_event, MOUSE_EVENT_BUTTON_NUMBER) as u32;
            let (name, raw) = other_mouse_button_name(btn);
            ("bu", name, raw)
        }
        _ => return None,
    };

    Some(RawInputEvent {
        time: SystemTime::now(),
        event_type: type_str,
        key,
        raw,
        name: None,
    })
}

fn other_mouse_button_name(button_number: u32) -> (String, u32) {
    match button_number {
        2 => ("Middle".to_string(), 3),
        n => (format!("Unknown({})", n + 1), n + 1),
    }
}

pub fn start_listener(
    tx: Sender<RawInputEvent>,
    recording: Arc<AtomicBool>,
) -> (std::thread::JoinHandle<()>, RunLoopRef) {
    let (rl_tx, rl_rx) = std::sync::mpsc::channel::<RunLoopRef>();

    let handle = std::thread::spawn(move || {
        unsafe {
            let pool = objc_autoreleasePoolPush();

            // Box the state so it has a stable address for the callback's user_info pointer.
            // It lives on the heap for the duration of CFRunLoopRun().
            let mut state = Box::new(ListenerState {
                tx,
                recording,
                last_flags: 0,
            });

            let tap = CGEventTapCreate(
                K_CG_HID_EVENT_TAP,
                K_CG_HEAD_INSERT_EVENT_TAP,
                K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
                EVENT_MASK,
                raw_callback,
                &mut *state as *mut ListenerState as *mut c_void,
            );

            if tap.is_null() {
                log::error!(
                    "Failed to create CGEventTap - accessibility permission may be missing"
                );
                let _ = rl_tx.send(RunLoopRef(std::ptr::null_mut()));
                objc_autoreleasePoolPop(pool);
                return;
            }

            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            if source.is_null() {
                log::error!("Failed to create CFRunLoopSource for CGEventTap");
                let _ = rl_tx.send(RunLoopRef(std::ptr::null_mut()));
                objc_autoreleasePoolPop(pool);
                return;
            }

            let current_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(current_loop, source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);

            // Retain the run loop ref so stop() can call CFRunLoopStop from another thread
            CFRetain(current_loop as *const c_void);
            let _ = rl_tx.send(RunLoopRef(current_loop));

            CFRunLoopRun();

            objc_autoreleasePoolPop(pool);
            // state is dropped here after the run loop exits
        }
    });

    let run_loop = rl_rx
        .recv()
        .expect("Failed to receive run loop ref from listener thread");
    (handle, run_loop)
}

/// Maps a macOS CGKeyCode to an rdev-compatible key name string.
/// Based on rdev's src/macos/keycodes.rs key_from_code mapping, with
/// ControlRight (62) and AltGr (61) fixes included.
fn key_name_from_code(code: u16) -> String {
    match code {
        0 => "KeyA",
        1 => "KeyS",
        2 => "KeyD",
        3 => "KeyF",
        4 => "KeyH",
        5 => "KeyG",
        6 => "KeyZ",
        7 => "KeyX",
        8 => "KeyC",
        9 => "KeyV",
        11 => "KeyB",
        12 => "KeyQ",
        13 => "KeyW",
        14 => "KeyE",
        15 => "KeyR",
        16 => "KeyY",
        17 => "KeyT",
        18 => "Num1",
        19 => "Num2",
        20 => "Num3",
        21 => "Num4",
        22 => "Num6",
        23 => "Num5",
        24 => "Equal",
        25 => "Num9",
        26 => "Num7",
        27 => "Minus",
        28 => "Num8",
        29 => "Num0",
        30 => "RightBracket",
        31 => "KeyO",
        32 => "KeyU",
        33 => "LeftBracket",
        34 => "KeyI",
        35 => "KeyP",
        36 => "Return",
        37 => "KeyL",
        38 => "KeyJ",
        39 => "Quote",
        40 => "KeyK",
        41 => "SemiColon",
        42 => "BackSlash",
        43 => "Comma",
        44 => "Slash",
        45 => "KeyN",
        46 => "KeyM",
        47 => "Dot",
        48 => "Tab",
        49 => "Space",
        50 => "BackQuote",
        51 => "Backspace",
        53 => "Escape",
        54 => "MetaRight",
        55 => "MetaLeft",
        56 => "ShiftLeft",
        57 => "CapsLock",
        58 => "Alt",
        59 => "ControlLeft",
        60 => "ShiftRight",
        61 => "AltGr",
        62 => "ControlRight",
        63 => "Function",
        96 => "F5",
        97 => "F6",
        98 => "F7",
        99 => "F3",
        100 => "F8",
        101 => "F9",
        103 => "F11",
        109 => "F10",
        111 => "F12",
        118 => "F4",
        120 => "F2",
        122 => "F1",
        123 => "LeftArrow",
        124 => "RightArrow",
        125 => "DownArrow",
        126 => "UpArrow",
        _ => return format!("Unknown({})", code),
    }
    .to_string()
}

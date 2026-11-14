//! Browser storage, diagnostics, installation, and lifecycle integration.

const STORAGE_KEY: &str = "gamevex.rust-demo.v1";
const QA_STORAGE_KEY: &str = "gamevex.rust-demo.qa.v1";

/// Loads the namespaced session, returning `None` when no saved session exists.
pub fn load() -> Result<Option<String>, String> {
    if qa_mode() == Some("unavailable") {
        return Err("QA: storage unavailable".into());
    }
    let window = web_sys::window().ok_or_else(|| "browser window is unavailable".to_owned())?;
    let storage = window
        .local_storage()
        .map_err(js_error)?
        .ok_or_else(|| "browser local storage is unavailable".to_owned())?;
    let key = storage_key();
    if qa_mode() == Some("corrupt") {
        if let Some(value) = storage.get_item(key).map_err(js_error)? {
            return Ok(Some(value));
        }
        let corrupt = "{ invalid GameVex save";
        if owns_save_writer() {
            storage.set_item(key, corrupt).map_err(js_error)?;
        }
        return Ok(Some(corrupt.into()));
    }
    storage.get_item(key).map_err(js_error)
}

/// Stores the serialized session. Callers should validate the data before handing it here.
pub fn store(contents: &str) -> Result<(), String> {
    if contents.len() > gamevex_core::save::MAX_SAVE_BYTES {
        return Err("saved session exceeds 256 KiB limit".into());
    }
    if qa_mode() == Some("unavailable") {
        return Err("QA: storage unavailable".into());
    }
    if !owns_save_writer() {
        return Err("save writer unavailable · close other arcade tabs and reload".into());
    }
    let window = web_sys::window().ok_or_else(|| "browser window is unavailable".to_owned())?;
    let storage = window
        .local_storage()
        .map_err(js_error)?
        .ok_or_else(|| "browser local storage is unavailable".to_owned())?;
    storage.set_item(storage_key(), contents).map_err(js_error)
}

/// Human-readable storage backend description for the settings/status panel.
pub fn description() -> String {
    match qa_mode() {
        Some(mode) => format!("Browser local storage ({QA_STORAGE_KEY}; QA: {mode})"),
        None => format!("Browser local storage ({STORAGE_KEY})"),
    }
}

/// Publishes the latest serialized save for browser diagnostics when explicitly requested.
/// The normal application URL never mirrors session contents into the DOM.
pub fn diagnostics_save(json: &str) {
    if diagnostics_enabled()
        && let Some(canvas) = canvas_element()
    {
        let _ = canvas.set_attribute("data-session", json);
    }
}

/// Keeps the most recent safe save available for synchronous browser lifecycle events.
pub fn stage_snapshot(save: &gamevex_core::Save, allowed: bool) {
    LIFECYCLE_SNAPSHOT.with(|snapshot| {
        *snapshot.borrow_mut() = (allowed && owns_save_writer()).then(|| save.clone());
    });
}

/// Timer ticks update the staged session without cloning boards or undo histories.
pub fn stage_elapsed(save: &gamevex_core::Save) {
    LIFECYCLE_SNAPSHOT.with(|snapshot| {
        if let Some(staged) = snapshot.borrow_mut().as_mut() {
            staged.minesweeper.game.elapsed_ms = save.minesweeper.game.elapsed_ms;
            staged.sudoku.game.elapsed_ms = save.sudoku.game.elapsed_ms;
            staged.mahjong.elapsed_ms = save.mahjong.elapsed_ms;
            staged.solitaire.elapsed_ms = save.solitaire.elapsed_ms;
            staged.freecell.elapsed_ms = save.freecell.elapsed_ms;
            staged.spider.elapsed_ms = save.spider.elapsed_ms;
            staged.backgammon.elapsed_ms = save.backgammon.elapsed_ms;
        }
    });
}

fn owns_save_writer() -> bool {
    canvas_element()
        .and_then(|canvas| canvas.get_attribute("data-save-writer"))
        .as_deref()
        == Some("true")
}

/// Installs app-lifetime handlers that flush the staged save before backgrounding or exit.
pub fn install_lifecycle_saves() {
    install_lifecycle_saves_wasm();
}

/// Whether the page/window is currently visible and can advance foreground-only behavior.
pub fn visible() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .map(|d| !d.hidden())
        .unwrap_or(false)
}

/// Returns the browser lifecycle generation.
pub fn lifecycle_epoch() -> u64 {
    canvas_element()
        .and_then(|canvas| canvas.get_attribute("data-lifecycle-epoch"))
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// Publishes the board bounds in Bevy logical coordinates for narrowly scoped browser input.
pub fn publish_board(bounds: Option<[f32; 4]>) {
    let Some(canvas) = canvas_element() else {
        return;
    };
    match bounds {
        Some(bounds) if bounds.iter().all(|value| value.is_finite()) => {
            if let Ok(json) = serde_json::to_string(&bounds) {
                let _ = canvas.set_attribute("data-board-bounds", &json);
                let _ = canvas.set_attribute("data-board-active", "true");
            }
        }
        _ => {
            let _ = canvas.set_attribute("data-board-active", "false");
            let _ = canvas.remove_attribute("data-board-bounds");
        }
    }
}

/// Signals that the WebAssembly app has initialized its canvas and render loop.
pub fn ready() {
    if let Some(canvas) = canvas_element() {
        let _ = canvas.set_attribute("data-ready", "true");
    }
}

fn canvas_element() -> Option<web_sys::HtmlCanvasElement> {
    use wasm_bindgen::JsCast;
    web_sys::window()?
        .document()?
        .get_element_by_id("gamevex")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()
}

fn query_parts() -> Option<String> {
    web_sys::window()?.location().search().ok()
}

fn qa_mode() -> Option<&'static str> {
    let query = query_parts()?;
    query
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.strip_prefix("qa="))
        .and_then(|value| match value {
            "corrupt" => Some("corrupt"),
            "unavailable" => Some("unavailable"),
            "fresh" => Some("fresh"),
            _ => None,
        })
}

pub fn diagnostics_enabled() -> bool {
    query_parts()
        .map(|query| {
            query
                .trim_start_matches('?')
                .split('&')
                .any(|part| part == "diagnostics")
        })
        .unwrap_or(false)
}

fn storage_key() -> &'static str {
    match qa_mode() {
        Some("corrupt" | "unavailable" | "fresh") => QA_STORAGE_KEY,
        _ => STORAGE_KEY,
    }
}

thread_local! {
    static LIFECYCLE_SNAPSHOT: std::cell::RefCell<Option<gamevex_core::Save>> = const { std::cell::RefCell::new(None) };
    static LIFECYCLE_LISTENERS: std::cell::RefCell<Vec<wasm_bindgen::closure::Closure<dyn FnMut()>>> = std::cell::RefCell::new(Vec::new());
}

fn install_lifecycle_saves_wasm() {
    use wasm_bindgen::{JsCast, closure::Closure};
    if LIFECYCLE_LISTENERS.with(|listeners| !listeners.borrow().is_empty()) {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let callback = Closure::<dyn FnMut()>::new(|| {
        LIFECYCLE_SNAPSHOT.with(|snapshot| {
            if let Some(save) = snapshot.borrow().as_ref()
                && let Ok(json) = save.to_json()
            {
                let _ = store(&json);
            }
        });
    });
    let function = callback.as_ref().unchecked_ref();
    let _ = window.add_event_listener_with_callback("blur", function);
    let _ = window.add_event_listener_with_callback("pagehide", function);
    let _ = document.add_event_listener_with_callback("visibilitychange", function);
    LIFECYCLE_LISTENERS.with(|listeners| listeners.borrow_mut().push(callback));
}

fn js_error(error: wasm_bindgen::JsValue) -> String {
    format!("browser storage error: {error:?}")
}

/// The browser canvas is laid out in CSS pixels. Winit may report a stale scale
/// during a monitor/emulated-DPI transition, so layout and picking share this source.
pub fn canvas_metrics() -> Option<[f32; 3]> {
    let canvas = canvas_element()?;
    let width = canvas.client_width() as f32;
    let height = canvas.client_height() as f32;
    let density = web_sys::window()?.device_pixel_ratio().clamp(0.5, 4.0) as f32;
    if width > 0.0 && height > 0.0 {
        Some([width, height, density])
    } else {
        None
    }
}

/// Publish the exact shader warp coefficient for scoped browser secondary clicks.
pub fn publish_monitor(curvature: f32) {
    if let Some(canvas) = canvas_element() {
        let _ = canvas.set_attribute("data-curvature", &curvature.to_string());
    }
}

/// Scope browser secondary-click behavior to cards that actually support flipping.
pub fn publish_cards(bounds: Option<[[f32; 4]; 5]>) {
    if let Some(canvas) = canvas_element() {
        if let Some(bounds) = bounds {
            if let Ok(json) = serde_json::to_string(&bounds) {
                let _ = canvas.set_attribute("data-card-bounds", &json);
            }
        } else {
            let _ = canvas.remove_attribute("data-card-bounds");
        }
    }
}

/// Opt-in state/layout inspection for browser regression checks.
pub fn diagnostics_attribute(name: &str, json: &str) {
    if diagnostics_enabled()
        && let Some(canvas) = canvas_element()
    {
        let _ = canvas.set_attribute(name, json);
    }
}
#[wasm_bindgen::prelude::wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = gamevexInstallAvailable)]
    fn browser_install_available() -> bool;
    #[wasm_bindgen(js_namespace = window, js_name = gamevexInstall)]
    fn browser_install();
}

pub fn install_available() -> bool {
    browser_install_available()
}

pub fn install() {
    browser_install();
}

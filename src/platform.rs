//! Small platform boundary for persisted sessions and foreground visibility.

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "gamevex.rust-demo.v1";
#[cfg(target_arch = "wasm32")]
const QA_STORAGE_KEY: &str = "gamevex.rust-demo.qa.v1";

/// Loads the namespaced session, returning `None` when no saved session exists.
pub fn load() -> Result<Option<String>, String> {
    #[cfg(target_arch = "wasm32")]
    {
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
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        let path = save_path()?;
        use std::io::Read;
        match std::fs::File::open(&path) {
            Ok(file) => {
                let metadata = file.metadata().map_err(|e| {
                    format!("could not inspect saved session {}: {e}", path.display())
                })?;
                if metadata.len() > gamevex_core::save::MAX_SAVE_BYTES as u64 {
                    return Err(format!(
                        "Invalid saved data: session exceeds 256 KiB limit: {}",
                        path.display()
                    ));
                }
                let mut bytes = Vec::with_capacity(metadata.len() as usize);
                file.take((gamevex_core::save::MAX_SAVE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|e| format!("could not read saved session {}: {e}", path.display()))?;
                if bytes.len() > gamevex_core::save::MAX_SAVE_BYTES {
                    return Err(format!(
                        "Invalid saved data: session exceeds 256 KiB limit: {}",
                        path.display()
                    ));
                }
                String::from_utf8(bytes)
                    .map(Some)
                    .map_err(|e| format!("Invalid saved data: session is not UTF-8: {e}"))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!(
                "could not read saved session {}: {e}",
                path.display()
            )),
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos")))]
    {
        Err("session persistence is unsupported on this native platform".into())
    }
}

/// Stores the serialized session. Callers should validate the data before handing it here.
pub fn store(contents: &str) -> Result<(), String> {
    if contents.len() > gamevex_core::save::MAX_SAVE_BYTES {
        return Err("saved session exceeds 256 KiB limit".into());
    }
    #[cfg(target_arch = "wasm32")]
    {
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
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        atomic_write(&save_path()?, contents.as_bytes())
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos")))]
    {
        Err("session persistence is unsupported on this native platform".into())
    }
}

/// Human-readable storage backend description for the settings/status panel.
pub fn description() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        match qa_mode() {
            Some(mode) => format!("Browser local storage ({QA_STORAGE_KEY}; QA: {mode})"),
            None => format!("Browser local storage ({STORAGE_KEY})"),
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        save_path()
            .map(|p| format!("{}", p.display()))
            .unwrap_or_else(|e| e)
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos")))]
    {
        "Session persistence unsupported on this native platform".into()
    }
}

/// Publishes the latest serialized save for browser diagnostics when explicitly requested.
/// The normal application URL never mirrors session contents into the DOM.
pub fn diagnostics_save(json: &str) {
    #[cfg(target_arch = "wasm32")]
    if diagnostics_enabled()
        && let Some(canvas) = canvas_element()
    {
        let _ = canvas.set_attribute("data-session", json);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = json;
    }
}

/// Keeps the most recent safe save available for synchronous browser lifecycle events.
pub fn stage_snapshot(save: &gamevex_core::Save, allowed: bool) {
    #[cfg(target_arch = "wasm32")]
    LIFECYCLE_SNAPSHOT.with(|snapshot| {
        *snapshot.borrow_mut() = (allowed && owns_save_writer()).then(|| save.clone());
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (save, allowed);
    }
}

/// Timer ticks update the staged session without cloning boards or undo histories.
pub fn stage_elapsed(save: &gamevex_core::Save) {
    #[cfg(target_arch = "wasm32")]
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
    #[cfg(not(target_arch = "wasm32"))]
    let _ = save;
}

#[cfg(target_arch = "wasm32")]
fn owns_save_writer() -> bool {
    canvas_element()
        .and_then(|canvas| canvas.get_attribute("data-save-writer"))
        .as_deref()
        == Some("true")
}

/// Installs app-lifetime handlers that flush the staged save before backgrounding or exit.
pub fn install_lifecycle_saves() {
    #[cfg(target_arch = "wasm32")]
    install_lifecycle_saves_wasm();
}

/// Whether the page/window is currently visible and can advance foreground-only behavior.
pub fn visible() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.document())
            .map(|d| !d.hidden())
            .unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

/// Returns the browser lifecycle generation. Native windows do not use this signal.
pub fn lifecycle_epoch() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        canvas_element()
            .and_then(|canvas| canvas.get_attribute("data-lifecycle-epoch"))
            .and_then(|value| value.parse().ok())
            .unwrap_or(0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// Publishes the board bounds in Bevy logical coordinates for narrowly scoped browser input.
pub fn publish_board(bounds: Option<[f32; 4]>) {
    #[cfg(target_arch = "wasm32")]
    {
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bounds;
    }
}

/// Signals that the WebAssembly app has initialized its canvas and render loop.
pub fn ready() {
    #[cfg(target_arch = "wasm32")]
    if let Some(canvas) = canvas_element() {
        let _ = canvas.set_attribute("data-ready", "true");
    }
}

#[cfg(target_arch = "wasm32")]
fn canvas_element() -> Option<web_sys::HtmlCanvasElement> {
    use wasm_bindgen::JsCast;
    web_sys::window()?
        .document()?
        .get_element_by_id("gamevex")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()
}

#[cfg(target_arch = "wasm32")]
fn query_parts() -> Option<String> {
    web_sys::window()?.location().search().ok()
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
fn storage_key() -> &'static str {
    match qa_mode() {
        Some("corrupt" | "unavailable" | "fresh") => QA_STORAGE_KEY,
        _ => STORAGE_KEY,
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static LIFECYCLE_SNAPSHOT: std::cell::RefCell<Option<gamevex_core::Save>> = const { std::cell::RefCell::new(None) };
    static LIFECYCLE_LISTENERS: std::cell::RefCell<Vec<wasm_bindgen::closure::Closure<dyn FnMut()>>> = std::cell::RefCell::new(Vec::new());
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn save_path() -> Result<std::path::PathBuf, String> {
    if let Some(override_dir) = std::env::var_os("GAMEVEX_SAVE_DIR") {
        return Ok(std::path::PathBuf::from(override_dir).join("session-v1.json"));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| "HOME is not set; cannot locate application support".to_owned())?;
    Ok(std::path::PathBuf::from(home)
        .join("Library/Application Support/GameVex Rust Demo/session-v1.json"))
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn atomic_write(path: &std::path::Path, contents: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .ok_or_else(|| "save path has no parent directory".to_owned())?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("could not create save directory {}: {e}", parent.display()))?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("system clock is before Unix epoch: {e}"))?
        .as_nanos();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut temp_name = path.file_name().unwrap_or_default().to_os_string();
    temp_name.push(format!(".{}.{nonce}.{sequence}.tmp", std::process::id()));
    let temp = parent.join(temp_name);
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("could not create temporary save {}: {e}", temp.display()))?;
        file.write_all(contents)
            .map_err(|e| format!("could not write temporary save: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("could not sync temporary save: {e}"))?;
        drop(file);
        std::fs::rename(&temp, path)
            .map_err(|e| format!("could not replace saved session {}: {e}", path.display()))?;
        std::fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| format!("could not sync save directory {}: {e}", parent.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}

#[cfg(target_arch = "wasm32")]
fn js_error(error: wasm_bindgen::JsValue) -> String {
    format!("browser storage error: {error:?}")
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_existing_content_and_leaves_no_temp_file() {
        let dir =
            std::env::temp_dir().join(format!("gamevex-platform-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("session-v1.json");
        atomic_write(&path, b"first").unwrap();
        atomic_write(&path, b"second").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn oversized_session_is_rejected_before_storage() {
        assert!(
            store(&"x".repeat(gamevex_core::save::MAX_SAVE_BYTES + 1))
                .unwrap_err()
                .contains("256 KiB")
        );
    }
}

/// The browser canvas is laid out in CSS pixels. Winit may report a stale scale
/// during a monitor/emulated-DPI transition, so layout and picking share this source.
pub fn canvas_metrics() -> Option<[f32; 3]> {
    #[cfg(target_arch = "wasm32")]
    {
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Publish the exact shader warp coefficient for scoped browser secondary clicks.
pub fn publish_monitor(_curvature: f32) {
    #[cfg(target_arch = "wasm32")]
    if let Some(canvas) = canvas_element() {
        let _ = canvas.set_attribute("data-curvature", &_curvature.to_string());
    }
}

/// Scope browser secondary-click behavior to cards that actually support flipping.
pub fn publish_cards(_bounds: Option<[[f32; 4]; 5]>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(canvas) = canvas_element() {
        if let Some(bounds) = _bounds {
            if let Ok(json) = serde_json::to_string(&bounds) {
                let _ = canvas.set_attribute("data-card-bounds", &json);
            }
        } else {
            let _ = canvas.remove_attribute("data-card-bounds");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn diagnostics_enabled() -> bool {
    false
}

/// Opt-in state/layout inspection for browser regression checks.
pub fn diagnostics_attribute(_name: &str, _json: &str) {
    #[cfg(target_arch = "wasm32")]
    if diagnostics_enabled()
        && let Some(canvas) = canvas_element()
    {
        let _ = canvas.set_attribute(_name, _json);
    }
}

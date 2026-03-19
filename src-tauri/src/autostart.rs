#[cfg(target_os = "windows")]
fn platform_enable(app: &tauri::AppHandle) -> Result<(), String> {
    const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_str = exe_path.to_string_lossy();

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(RUN_KEY)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;
    key.set_value(
        app.package_info().name.clone(),
        &format!("\"{}\"", exe_str),
    )
    .map_err(|e| format!("Failed to set registry value: {}", e))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_disable(app: &tauri::AppHandle) -> Result<(), String> {
    const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(RUN_KEY, winreg::enums::KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;
    match key.delete_value(app.package_info().name.clone()) {
        Ok(()) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to delete registry value: {}", e)),
    }
}

#[cfg(target_os = "windows")]
fn platform_is_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let key = match hkcu.open_subkey(RUN_KEY) {
        Ok(k) => k,
        Err(_) => return Ok(false),
    };
    let result: Result<String, _> = key.get_value(app.package_info().name.clone());
    Ok(result.is_ok())
}

#[cfg(target_os = "macos")]
fn platform_enable(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .enable()
        .map_err(|e| format!("Failed to enable autostart: {}", e))
}

#[cfg(target_os = "macos")]
fn platform_disable(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .disable()
        .map_err(|e| format!("Failed to disable autostart: {}", e))
}

#[cfg(target_os = "macos")]
fn platform_is_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .is_enabled()
        .map_err(|e| format!("Failed to check autostart: {}", e))
}

#[tauri::command]
pub fn enable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    platform_enable(&app)
}

#[tauri::command]
pub fn disable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    platform_disable(&app)
}

#[tauri::command]
pub fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    platform_is_enabled(&app)
}

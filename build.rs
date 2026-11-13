// ClipVault build script: embed version info + brand icon (Windows only).
// Requires rc.exe from the Windows SDK. On any failure we warn and continue —
// the app still builds and the tray icon is generated at runtime.
fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let ico = std::path::Path::new(&dir).join("assets").join("icon.ico");
            if ico.exists() {
                if let Some(s) = ico.to_str() {
                    res.set_icon(s);
                }
            }
        }
        res.set("ProductName", "ClipVault")
            .set("FileDescription", "ClipVault - private clipboard manager")
            .set("CompanyName", "Manuel")
            .set("LegalCopyright", "MIT")
            .set("FileVersion", "0.1.0.0")
            .set("ProductVersion", "0.1.0.0")
            .set_language(0x0409);
        if let Err(e) = res.compile() {
            println!("cargo:warning=winres skipped: {e}");
        }
    }
}

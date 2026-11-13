// Emit the ClipVault brand icon to assets/icon.ico (run once at packaging time).
fn main() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("icon.ico");
    match clipvault::platform::icon::write_ico(&path) {
        Ok(()) => println!("wrote {}", path.display()),
        Err(e) => eprintln!("failed: {e}"),
    }
}

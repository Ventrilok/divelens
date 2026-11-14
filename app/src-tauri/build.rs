fn main() {
    // The app's version comes from app/package.json (tauri.conf.json points
    // to it), which tauri-build does not watch: rebuild when it changes.
    println!("cargo:rerun-if-changed=../package.json");
    tauri_build::build()
}

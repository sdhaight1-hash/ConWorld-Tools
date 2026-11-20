fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["platform_scene", "complete_platform_smoke"]),
    ))
    .expect("failed to build the desktop shell configuration");
}

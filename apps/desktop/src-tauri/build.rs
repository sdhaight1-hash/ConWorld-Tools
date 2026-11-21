fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "platform_scene",
            "complete_platform_smoke",
            "northbridge_history",
            "workspace_worlds",
            "workspace_create_world",
            "workspace_snapshot",
            "workspace_search",
            "workspace_create_person",
            "workspace_create_relationship",
            "workspace_record_relationship_state",
            "workspace_save_definition",
            "workspace_evidence",
            "workspace_record_evidence",
            "workspace_record_event",
        ]),
    ))
    .expect("failed to build the desktop shell configuration");
}

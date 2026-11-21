#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod history;
mod workspace;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

#[derive(Deserialize, Serialize)]
struct Scene {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Deserialize, Serialize)]
struct Node {
    id: String,
    label: String,
    kind: String,
    x: u16,
    y: u16,
}

#[derive(Deserialize, Serialize)]
struct Edge {
    source: String,
    target: String,
    label: String,
}

struct PlatformSmoke(bool);

#[derive(Serialize)]
struct PlatformScene {
    target: &'static str,
    smoke: bool,
    scene: Scene,
}

#[derive(Deserialize, Serialize)]
struct SmokeObservation {
    scene_verified: bool,
    canvas2d_verified: bool,
    history_verified: bool,
    webgpu: WebGpuStatus,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum WebGpuStatus {
    Available,
    ApiUnavailable,
    AdapterUnavailable,
    ProbeFailed,
    TimedOut,
}

fn load_scene(smoke: bool) -> Result<PlatformScene, &'static str> {
    let scene = serde_json::from_str(include_str!("../../src/northbridge.json"))
        .map_err(|_| "The bundled Northbridge scene is invalid.")?;
    Ok(PlatformScene {
        target: std::env::consts::OS,
        smoke,
        scene,
    })
}

#[tauri::command]
fn platform_scene(smoke: State<'_, PlatformSmoke>) -> Result<PlatformScene, &'static str> {
    load_scene(smoke.0)
}

#[tauri::command]
fn complete_platform_smoke(
    app: AppHandle,
    smoke: State<'_, PlatformSmoke>,
    observation: SmokeObservation,
) -> Result<(), &'static str> {
    if !smoke.0 {
        return Err("Platform smoke mode is disabled.");
    }
    let webview_version =
        tauri::webview_version().map_err(|_| "Cannot identify the native webview.")?;
    let report = serde_json::json!({
        "target": std::env::consts::OS,
        "webview_version": webview_version,
        "observation": observation,
    });
    println!("ATLAS_PLATFORM_SMOKE {report}");
    app.exit(i32::from(
        !observation.scene_verified
            || !observation.canvas2d_verified
            || !observation.history_verified,
    ));
    Ok(())
}

fn main() {
    let smoke = std::env::args_os().any(|arg| arg == "--platform-smoke");
    let history = history::NorthbridgeHistory::seed()
        .expect("failed to initialize the Phase 2 Northbridge history fixture");
    tauri::Builder::default()
        .manage(PlatformSmoke(smoke))
        .manage(history)
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            app.manage(workspace::AuthorWorkspace::open(directory.join("worlds.sqlite"))?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform_scene,
            complete_platform_smoke,
            history::northbridge_history,
            workspace::workspace_worlds,
            workspace::workspace_create_world,
            workspace::workspace_snapshot,
            workspace::workspace_create_person,
            workspace::workspace_create_relationship,
            workspace::workspace_record_event
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Living History Atlas");
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn bundled_scene_has_unique_nodes_and_resolvable_edges() {
        let scene = super::load_scene(false).unwrap().scene;
        let ids: HashSet<_> = scene.nodes.iter().map(|node| &node.id).collect();
        assert!(!ids.is_empty());
        assert_eq!(ids.len(), scene.nodes.len());
        assert!(
            scene
                .edges
                .iter()
                .all(|edge| { ids.contains(&edge.source) && ids.contains(&edge.target) })
        );
    }
}

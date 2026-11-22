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

#[derive(Clone, Copy)]
enum PlatformSmoke {
    None,
    Scene,
    Renderer { require_gpu: bool },
}

#[derive(Serialize)]
struct PlatformScene {
    target: &'static str,
    smoke: bool,
    renderer_smoke: bool,
    scene: Scene,
}

#[derive(Deserialize, Serialize)]
struct SmokeObservation {
    scene_verified: bool,
    canvas2d_verified: bool,
    history_verified: bool,
    webgpu: WebGpuStatus,
    benchmark: Option<BenchmarkObservation>,
}

#[derive(Deserialize, Serialize)]
struct BenchmarkObservation {
    adapter_fallback: Option<bool>,
    adapter_vendor: Option<String>,
    adapter_architecture: Option<String>,
    nodes: usize,
    edges: usize,
    view_preserved: bool,
    fallback_verified: bool,
    gpu_pixels_verified: bool,
    device_loss_verified: bool,
    recovery_verified: bool,
    frame_intervals_ms: Vec<f64>,
}

impl BenchmarkObservation {
    fn passed(&self, require_gpu: bool, webgpu: &WebGpuStatus) -> bool {
        let gpu_verified = self.gpu_pixels_verified
            && self.device_loss_verified
            && self.recovery_verified
            && self.frame_intervals_ms.len() == 24
            && self
                .frame_intervals_ms
                .iter()
                .all(|sample| sample.is_finite() && *sample > 0.0);
        let unsupported = matches!(
            webgpu,
            WebGpuStatus::ApiUnavailable | WebGpuStatus::AdapterUnavailable
        );
        self.nodes == 2_000
            && self.edges == 3_910
            && self.view_preserved
            && self.fallback_verified
            && (gpu_verified
                && matches!(webgpu, WebGpuStatus::Available)
                && (!require_gpu || self.adapter_fallback == Some(false))
                || !require_gpu && unsupported)
    }
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

fn load_scene(mode: PlatformSmoke) -> Result<PlatformScene, &'static str> {
    let scene = serde_json::from_str(include_str!("../../src/northbridge.json"))
        .map_err(|_| "The bundled Northbridge scene is invalid.")?;
    Ok(PlatformScene {
        target: std::env::consts::OS,
        smoke: !matches!(mode, PlatformSmoke::None),
        renderer_smoke: matches!(mode, PlatformSmoke::Renderer { .. }),
        scene,
    })
}

#[tauri::command]
fn platform_scene(smoke: State<'_, PlatformSmoke>) -> Result<PlatformScene, &'static str> {
    load_scene(*smoke)
}

#[tauri::command]
fn complete_platform_smoke(
    app: AppHandle,
    smoke: State<'_, PlatformSmoke>,
    observation: SmokeObservation,
) -> Result<(), &'static str> {
    let passed = match *smoke {
        PlatformSmoke::None => return Err("Platform smoke mode is disabled."),
        PlatformSmoke::Scene => {
            observation.scene_verified
                && observation.canvas2d_verified
                && observation.history_verified
        }
        PlatformSmoke::Renderer { require_gpu } => observation
            .benchmark
            .as_ref()
            .is_some_and(|benchmark| benchmark.passed(require_gpu, &observation.webgpu)),
    };
    let webview_version =
        tauri::webview_version().map_err(|_| "Cannot identify the native webview.")?;
    let report = serde_json::json!({
        "target": std::env::consts::OS,
        "webview_version": webview_version,
        "observation": observation,
    });
    println!("ATLAS_PLATFORM_SMOKE {report}");
    app.exit(i32::from(!passed));
    Ok(())
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--restore-archive")) {
        let (Some(source), Some(destination), None) = (args.next(), args.next(), args.next())
        else {
            eprintln!("Usage: atlas-desktop --restore-archive BACKUP.sqlite NEW.sqlite");
            std::process::exit(2);
        };
        match atlas_store::archive::restore_archive(
            source.as_ref(),
            destination.as_ref(),
            atlas_store::Scope::Author,
        ) {
            Ok(manifest) => println!(
                "Restored {} world(s) to {}",
                manifest.worlds.len(),
                destination.to_string_lossy()
            ),
            Err(error) => {
                eprintln!("Restore failed: {error}");
                std::process::exit(1);
            }
        }
        return;
    }
    let has_flag = |name| std::env::args_os().any(|arg| arg == name);
    let smoke = if has_flag("--renderer-smoke") {
        PlatformSmoke::Renderer {
            require_gpu: has_flag("--require-gpu"),
        }
    } else if has_flag("--platform-smoke") {
        PlatformSmoke::Scene
    } else {
        PlatformSmoke::None
    };
    let history = history::NorthbridgeHistory::seed()
        .expect("failed to initialize the Phase 2 Northbridge history fixture");
    tauri::Builder::default()
        .manage(smoke)
        .manage(history)
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            app.manage(workspace::AuthorWorkspace::open(
                directory.join("worlds.sqlite"),
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform_scene,
            complete_platform_smoke,
            history::northbridge_history,
            workspace::workspace_worlds,
            workspace::workspace_create_world,
            workspace::workspace_snapshot,
            workspace::workspace_search,
            workspace::workspace_next_search_page,
            workspace::workspace_cancel_query,
            workspace::workspace_discover_mentions,
            workspace::workspace_review_mention,
            workspace::workspace_save_person_alias,
            workspace::workspace_saved_queries,
            workspace::workspace_save_query,
            workspace::workspace_replay_saved_query,
            workspace::workspace_query_path,
            workspace::workspace_query_participant_events,
            workspace::workspace_composite_query,
            workspace::workspace_family_match,
            workspace::workspace_family_links,
            workspace::workspace_record_parentage,
            workspace::workspace_withdraw_parentage,
            workspace::workspace_entry_layout,
            workspace::workspace_preview_author_layout,
            workspace::workspace_save_layout,
            workspace::workspace_public_entry,
            workspace::workspace_author_entry_backlinks,
            workspace::workspace_public_entry_backlinks,
            workspace::workspace_search_public_entry,
            workspace::workspace_search_public_pages,
            workspace::workspace_custom_templates,
            workspace::workspace_save_custom_template,
            workspace::workspace_instantiate_custom_template,
            workspace::workspace_ghost_candidates,
            workspace::workspace_resolve_ghost,
            workspace::workspace_founder_genome,
            workspace::workspace_create_identity,
            workspace::workspace_network_facts,
            workspace::workspace_persisted_impact,
            workspace::workspace_record_network_fact,
            workspace::workspace_withdraw_network_fact,
            workspace::workspace_propose_ghost,
            workspace::workspace_ghost_proposals,
            workspace::workspace_review_ghost,
            workspace::workspace_create_person,
            workspace::workspace_create_relationship,
            workspace::workspace_record_relationship_state,
            workspace::workspace_save_definition,
            workspace::workspace_evidence,
            workspace::workspace_record_evidence,
            workspace::workspace_record_event
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Living History Atlas");
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn renderer_smoke_requires_pixels_loss_recovery_and_timings_on_gpu() {
        use super::{BenchmarkObservation, WebGpuStatus};
        let mut benchmark = BenchmarkObservation {
            adapter_fallback: None,
            adapter_vendor: None,
            adapter_architecture: None,
            nodes: 2_000,
            edges: 3_910,
            view_preserved: true,
            fallback_verified: true,
            gpu_pixels_verified: false,
            device_loss_verified: false,
            recovery_verified: false,
            frame_intervals_ms: vec![],
        };
        assert!(benchmark.passed(false, &WebGpuStatus::AdapterUnavailable));
        assert!(!benchmark.passed(true, &WebGpuStatus::AdapterUnavailable));
        assert!(!benchmark.passed(false, &WebGpuStatus::Available));
        assert!(!benchmark.passed(false, &WebGpuStatus::TimedOut));
        benchmark.gpu_pixels_verified = true;
        benchmark.device_loss_verified = true;
        benchmark.recovery_verified = true;
        benchmark.frame_intervals_ms = vec![16.0; 24];
        assert!(!benchmark.passed(true, &WebGpuStatus::Available));
        benchmark.adapter_fallback = Some(true);
        assert!(!benchmark.passed(true, &WebGpuStatus::Available));
        benchmark.adapter_fallback = Some(false);
        assert!(benchmark.passed(true, &WebGpuStatus::Available));
        benchmark.view_preserved = false;
        assert!(!benchmark.passed(true, &WebGpuStatus::Available));
    }

    #[test]
    fn bundled_scene_has_unique_nodes_and_resolvable_edges() {
        let scene = super::load_scene(super::PlatformSmoke::None).unwrap().scene;
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

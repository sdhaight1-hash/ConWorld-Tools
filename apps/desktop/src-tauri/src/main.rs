#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct PlatformScene {
    target: &'static str,
    scene: Scene,
}

#[tauri::command]
fn platform_scene() -> Result<PlatformScene, &'static str> {
    let scene = serde_json::from_str(include_str!("../../src/northbridge.json"))
        .map_err(|_| "The bundled Northbridge scene is invalid.")?;
    Ok(PlatformScene {
        target: std::env::consts::OS,
        scene,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![platform_scene])
        .run(tauri::generate_context!())
        .expect("failed to start Living History Atlas");
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn bundled_scene_has_unique_nodes_and_resolvable_edges() {
        let scene = super::platform_scene().unwrap().scene;
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

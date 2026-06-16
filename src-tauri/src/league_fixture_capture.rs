//! Helpers for capturing live League API responses as JSON fixtures.
//!
//! Fixtures are written under `{workspace}/test-data/league/{mode_slug}/{phase}/`.
//! Enable capture with `WINGBOT_CAPTURE_FIXTURES=1`.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameModeContext {
    pub slug: String,
    pub queue_id: Option<u64>,
    pub game_mode: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FixtureMeta {
    pub captured_at: String,
    pub game_mode_slug: String,
    pub queue_id: Option<u64>,
    pub game_mode: Option<String>,
    pub phase: String,
    pub gameflow_phase: Option<String>,
    pub source: String,
}

impl FixtureMeta {
    pub fn wrap(&self, data: &Value) -> Value {
        json!({
            "meta": {
                "captured_at": self.captured_at,
                "game_mode_slug": self.game_mode_slug,
                "queue_id": self.queue_id,
                "game_mode": self.game_mode,
                "phase": self.phase,
                "gameflow_phase": self.gameflow_phase,
                "source": self.source,
            },
            "data": data,
        })
    }
}

pub fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test-data/league")
}

pub fn capture_enabled() -> bool {
    matches!(
        std::env::var("WINGBOT_CAPTURE_FIXTURES").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

pub fn queue_id_to_slug(queue_id: u64) -> Option<&'static str> {
    match queue_id {
        400 | 480 => Some("draft"),
        420 => Some("ranked_solo"),
        430 => Some("blind"),
        440 => Some("ranked_flex"),
        450 => Some("aram"),
        2400 => Some("aram_mayhem"),
        700 => Some("clash"),
        830 | 840 | 850 => Some("coop_ai"),
        900 | 1900 => Some("urf"),
        1020 => Some("one_for_all"),
        1300 => Some("nexus_blitz"),
        1400 => Some("ultbook"),
        1700 => Some("arena"),
        _ => None,
    }
}

pub fn game_mode_to_slug(game_mode: &str) -> &'static str {
    match game_mode.to_ascii_uppercase().as_str() {
        "CLASSIC" => "classic",
        "ARAM" => "aram",
        "KIWI" => "aram_mayhem",
        "URF" => "urf",
        "ONEFORALL" | "ONE_FOR_ALL" => "one_for_all",
        "NEXUSBLITZ" | "NEXUS_BLITZ" => "nexus_blitz",
        "ULTBOOK" | "ULTIMATE_SPELLBOOK" => "ultbook",
        "ARENA" => "arena",
        "TUTORIAL" => "tutorial",
        _ => "other",
    }
}

pub fn resolve_game_mode_context(
    queue_id: Option<u64>,
    game_mode: Option<&str>,
) -> GameModeContext {
    if let Some(id) = queue_id {
        if let Some(slug) = queue_id_to_slug(id) {
            return GameModeContext {
                slug: slug.to_string(),
                queue_id: Some(id),
                game_mode: game_mode.map(str::to_string),
            };
        }
    }

    if let Some(mode) = game_mode.filter(|m| !m.is_empty()) {
        return GameModeContext {
            slug: game_mode_to_slug(mode).to_string(),
            queue_id,
            game_mode: Some(mode.to_string()),
        };
    }

    GameModeContext {
        slug: "unknown".to_string(),
        queue_id,
        game_mode: None,
    }
}

pub fn extract_queue_id(value: &Value) -> Option<u64> {
    const KEYS: [&str; 3] = ["queueId", "queueID", "id"];

    for key in KEYS {
        if let Some(id) = value.get(key).and_then(|v| v.as_u64()) {
            return Some(id);
        }
    }

    value
        .get("queue")
        .and_then(|q| q.get("id"))
        .and_then(|v| v.as_u64())
}

pub fn extract_game_mode(value: &Value) -> Option<String> {
    value
        .get("gameMode")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            value
                .get("queue")
                .and_then(|q| q.get("gameMode"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
}

pub fn context_from_gameflow_session(session: &Value) -> GameModeContext {
    let game_data = session.get("gameData");
    let queue_id = game_data.and_then(extract_queue_id);
    let game_mode = game_data.and_then(extract_game_mode);
    resolve_game_mode_context(queue_id, game_mode.as_deref())
}

pub fn context_from_champion_select(session: &Value) -> GameModeContext {
    let queue_id = extract_queue_id(session);
    resolve_game_mode_context(queue_id, None)
}

pub fn context_from_live_game(all_game_data: &Value) -> GameModeContext {
    let game_data = all_game_data.get("gameData");
    let queue_id = game_data.and_then(extract_queue_id);
    let game_mode = game_data.and_then(extract_game_mode);
    resolve_game_mode_context(queue_id, game_mode.as_deref())
}

pub fn phase_dir(slug: &str, phase: &str) -> PathBuf {
    fixtures_root().join(slug).join(phase)
}

pub fn save_fixture(
    dir: &Path,
    filename: &str,
    meta: &FixtureMeta,
    data: &Value,
) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|e| format!("create fixture dir {}: {e}", dir.display()))?;

    let path = dir.join(filename);
    let wrapped = meta.wrap(data);
    let pretty = serde_json::to_string_pretty(&wrapped)
        .map_err(|e| format!("serialize fixture {}: {e}", path.display()))?;
    fs::write(&path, pretty).map_err(|e| format!("write fixture {}: {e}", path.display()))?;
    Ok(path)
}

pub fn update_capture_manifest(dir: &Path, entries: &[(String, PathBuf)]) -> Result<(), String> {
    let mut files = serde_json::Map::new();
    for (name, path) in entries {
        files.insert(
            name.clone(),
            Value::String(path.strip_prefix(fixtures_root()).unwrap_or(path).display().to_string()),
        );
    }

    let manifest = json!({
        "updated_at": chrono::Utc::now().to_rfc3339(),
        "files": files,
    });

    let path = dir.join("manifest.json");
    let pretty = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("serialize manifest: {e}"))?;
    fs::write(&path, pretty).map_err(|e| format!("write manifest {}: {e}", path.display()))?;
    Ok(())
}

pub fn list_fixture_files(filename: &str) -> Vec<PathBuf> {
    let root = fixtures_root();
    if !root.exists() {
        return Vec::new();
    }

    let mut paths = Vec::new();
    collect_fixture_files(&root, Path::new(""), filename, &mut paths);
    paths.sort();
    paths
}

fn collect_fixture_files(current: &Path, relative: &Path, filename: &str, out: &mut Vec<PathBuf>) {
    let read_dir = match fs::read_dir(current) {
        Ok(d) => d,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let next = relative.join(path.file_name().unwrap_or_default());
            collect_fixture_files(&path, &next, filename, out);
            continue;
        }

        if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            out.push(relative.join(filename));
        }
    }
}

pub fn load_fixture_data(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("read fixture {}: {e}", path.display()))?;
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse fixture {}: {e}", path.display()))?;

    Ok(parsed
        .get("data")
        .cloned()
        .unwrap_or(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_aram_from_queue_id() {
        let ctx = resolve_game_mode_context(Some(450), None);
        assert_eq!(ctx.slug, "aram");
        assert_eq!(ctx.queue_id, Some(450));
    }

    #[test]
    fn resolves_classic_from_game_mode() {
        let ctx = resolve_game_mode_context(None, Some("CLASSIC"));
        assert_eq!(ctx.slug, "classic");
    }

    #[test]
    fn extracts_nested_gameflow_queue() {
        let session = json!({
            "phase": "ChampSelect",
            "gameData": {
                "queue": { "id": 450, "gameMode": "ARAM" }
            }
        });
        let ctx = context_from_gameflow_session(&session);
        assert_eq!(ctx.slug, "aram");
        assert_eq!(ctx.queue_id, Some(450));
    }
}

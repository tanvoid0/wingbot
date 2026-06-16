//! Live League Client API tests — run against a real client while logged in.
//!
//! ```powershell
//! cd src-tauri
//! cargo test --lib league_live -- --nocapture --test-threads=1
//! ```
//!
//! Phase-specific checks skip automatically when you're in the wrong phase.
//! Re-run during champ select and again once in game to exercise both paths.
//!
//! Capture JSON fixtures for parsing (ARAM, SR, URF, etc.):
//!
//! ```powershell
//! cd src-tauri
//! $env:WINGBOT_CAPTURE_FIXTURES=1
//! cargo test --lib capture_fixtures -- --nocapture --test-threads=1
//! ```
//!
//! Fixtures are saved to `test-data/league/{mode_slug}/{phase}/` with a `meta`
//! wrapper describing queue, mode, and capture time. Re-run in ARAM champ select,
//! then again once the ARAM game loads, to seed both phases.

#[cfg(test)]
mod sanity {
    use crate::lcu_api::LCUAPI;
    use crate::types::GameSnapshot;
    use serde_json::{json, Value};

    pub fn check_lcu_connection(conn: &crate::lcu_api::LCUConnection) -> Result<(), String> {
        if conn.port == 0 {
            return Err("LCU port must not be 0".into());
        }
        if conn.password.is_empty() {
            return Err("LCU password must not be empty".into());
        }
        if conn.protocol != "https" {
            return Err(format!("unexpected LCU protocol: {}", conn.protocol));
        }
        Ok(())
    }

    pub fn check_summoner(summoner: &Value) -> Result<(), String> {
        let has_name = summoner.get("displayName").is_some()
            || summoner.get("gameName").is_some()
            || summoner.get("internalName").is_some();
        if !has_name {
            return Err("summoner missing displayName/gameName/internalName".into());
        }
        if summoner.get("puuid").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
            return Err("summoner missing puuid".into());
        }
        Ok(())
    }

    pub fn check_gameflow_phase(phase: &str) -> Result<(), String> {
        if phase.is_empty() {
            return Err("gameflow phase is empty".into());
        }
        Ok(())
    }

    pub fn check_champion_select_session(data: &Value) -> Result<(), String> {
        if !LCUAPI::is_champion_select_session(data) {
            return Err("payload is not a champion select session".into());
        }
        let my_team = data
            .get("myTeam")
            .and_then(|v| v.as_array())
            .ok_or("champion select missing myTeam array")?;
        if my_team.is_empty() {
            return Err("champion select myTeam is empty".into());
        }
        let phase = data
            .get("timer")
            .and_then(|t| t.get("phase"))
            .and_then(|p| p.as_str())
            .ok_or("champion select missing timer.phase")?;
        if phase.is_empty() {
            return Err("champion select timer.phase is empty".into());
        }
        Ok(())
    }

    pub fn check_live_game_data(data: &Value) -> Result<(), String> {
        let game_data = data
            .get("gameData")
            .ok_or("live game missing gameData")?;
        game_data
            .get("gameMode")
            .filter(|v| !v.is_null())
            .ok_or("live game missing gameData.gameMode")?;

        let players = data
            .get("allPlayers")
            .and_then(|v| v.as_array())
            .ok_or("live game missing allPlayers array")?;
        if players.len() < 2 {
            return Err(format!(
                "live game expected at least 2 players, got {}",
                players.len()
            ));
        }
        for (i, player) in players.iter().enumerate() {
            let name = player
                .get("championName")
                .or_else(|| player.get("rawChampionName"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if name.is_empty() {
                return Err(format!("player {} missing championName", i));
            }
            player
                .get("team")
                .filter(|v| !v.is_null())
                .ok_or_else(|| format!("player {} missing team", i))?;
        }
        Ok(())
    }

    pub fn check_game_snapshot(snapshot: &GameSnapshot) -> Result<(), String> {
        if !snapshot.lcu_connected {
            return Err("snapshot reports LCU disconnected".into());
        }
        check_gameflow_phase(&snapshot.game_phase)?;

        if snapshot.timestamp == 0 {
            return Err("snapshot timestamp is 0".into());
        }

        if snapshot.game_phase == "ChampSelect" {
            let cs = snapshot
                .champion_select
                .as_ref()
                .ok_or("ChampSelect phase but champion_select is None")?;
            check_champion_select_session(cs)?;
        }

        if snapshot.game_phase == "InProgress"
            && snapshot.live_client_available
            && !snapshot.is_spectator_mode
        {
            let live = snapshot
                .live_game
                .as_ref()
                .ok_or("InProgress with live client but live_game is None")?;
            check_live_game_data(live)?;
            snapshot
                .live_stats
                .as_ref()
                .ok_or("InProgress with live client but live_stats is None")?;
        }

        Ok(())
    }

    #[test]
    fn test_offline_champion_select_sanity() {
        let data = json!({
            "myTeam": [{ "cellId": 0, "summonerId": 1 }],
            "timer": { "phase": "BAN_PICK", "adjustedTimeLeftInPhase": 30000 }
        });
        assert!(check_champion_select_session(&data).is_ok());
    }

    #[test]
    fn test_offline_live_game_sanity() {
        let data = json!({
            "gameData": { "gameMode": "CLASSIC", "gameTime": 120.5 },
            "allPlayers": [
                { "championName": "Ahri", "team": "ORDER", "summonerName": "Player1" },
                { "championName": "Garen", "team": "CHAOS", "summonerName": "Player2" }
            ]
        });
        assert!(check_live_game_data(&data).is_ok());
    }

    #[test]
    fn test_offline_rejects_invalid_champion_select() {
        let data = json!({ "phase": "BAN_PICK" });
        assert!(check_champion_select_session(&data).is_err());
    }

    #[test]
    fn test_offline_rejects_invalid_live_game() {
        let data = json!({ "gameData": { "gameMode": "CLASSIC" } });
        assert!(check_live_game_data(&data).is_err());
    }
}

#[cfg(test)]
mod league_live {
    use super::sanity::{
        check_champion_select_session, check_game_snapshot, check_gameflow_phase,
        check_live_game_data, check_lcu_connection, check_summoner,
    };
    use crate::lcu_api::LCUAPI;

    async fn connect_live_api() -> Result<LCUAPI, String> {
        let mut api = LCUAPI::new();
        let connection = api.detect_connection().await?;
        check_lcu_connection(&connection)?;
        api.set_connection(connection);
        Ok(api)
    }

    async fn current_phase(api: &LCUAPI) -> Result<String, String> {
        api.get_current_game_state().await
    }

    fn skip_msg(label: &str, reason: &str) {
        println!("[SKIP] {label}: {reason}");
    }

    #[tokio::test]
    async fn test_live_detect_league_client() {
        println!("Detecting League client...");
        match connect_live_api().await {
            Ok(api) => {
                let conn = api.connection.as_ref().unwrap();
                println!(
                    "LCU connected on port {} ({})",
                    conn.port, conn.protocol
                );
                assert!(api.has_connection());
            }
            Err(e) => {
                panic!(
                    "League client not detected: {e}. Open League and log in, then re-run."
                );
            }
        }
    }

    #[tokio::test]
    async fn test_live_lcu_baseline_apis() {
        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = api
            .get_current_game_state()
            .await
            .expect("gameflow-phase should respond");
        check_gameflow_phase(&phase).expect("invalid gameflow phase");
        println!("Gameflow phase: {phase}");

        let summoner = api
            .get_current_summoner()
            .await
            .expect("current-summoner should respond");
        check_summoner(&summoner).expect("summoner data failed sanity check");
        println!(
            "Summoner: {}",
            summoner
                .get("gameName")
                .or_else(|| summoner.get("displayName"))
                .unwrap_or(&serde_json::Value::Null)
        );

        let health = api
            .check_connection_health()
            .await
            .expect("connection health check should succeed");
        assert!(
            health.get("lcu_connected").and_then(|v| v.as_bool()) == Some(true),
            "health should report LCU connected"
        );
        println!("Connection health: {health}");
    }

    #[tokio::test]
    async fn test_live_game_snapshot() {
        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let snapshot = api
            .get_game_snapshot()
            .await
            .expect("get_game_snapshot should succeed");

        println!(
            "Snapshot: phase={}, live_client={}, spectator={}, cs={}, live_game={}",
            snapshot.game_phase,
            snapshot.live_client_available,
            snapshot.is_spectator_mode,
            snapshot.champion_select.is_some(),
            snapshot.live_game.is_some()
        );

        check_game_snapshot(&snapshot).expect("game snapshot failed sanity check");
    }

    #[tokio::test]
    async fn test_live_champion_select_apis() {
        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = match current_phase(&api).await {
            Ok(p) => p,
            Err(e) => {
                skip_msg("champion_select", &format!("cannot read phase: {e}"));
                return;
            }
        };

        if phase != "ChampSelect" {
            skip_msg(
                "champion_select",
                &format!("current phase is {phase}, need ChampSelect"),
            );
            return;
        }

        println!("In ChampSelect — validating champion select APIs...");

        let cs = api
            .get_champion_select_state()
            .await
            .expect("champ-select session should respond during ChampSelect");
        check_champion_select_session(&cs).expect("champion select data failed sanity check");

        let my_team = cs.get("myTeam").and_then(|v| v.as_array()).unwrap();
        let timer_phase = cs
            .get("timer")
            .and_then(|t| t.get("phase"))
            .and_then(|p| p.as_str())
            .unwrap_or("?");
        println!(
            "Champ select OK: myTeam={}, timer.phase={}",
            my_team.len(),
            timer_phase
        );

        let snapshot = api.get_game_snapshot().await.expect("snapshot");
        assert!(
            snapshot.champion_select.is_some(),
            "snapshot should include champion_select during ChampSelect"
        );
        assert_eq!(snapshot.game_phase, "ChampSelect");
    }

    #[tokio::test]
    async fn test_live_in_game_apis() {
        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = match current_phase(&api).await {
            Ok(p) => p,
            Err(e) => {
                skip_msg("in_game", &format!("cannot read phase: {e}"));
                return;
            }
        };

        if phase != "InProgress" {
            skip_msg(
                "in_game",
                &format!("current phase is {phase}, need InProgress"),
            );
            return;
        }

        println!("In game — validating live client APIs...");

        assert!(
            api.is_live_client_available().await,
            "live client (port 2999) should be available during InProgress"
        );

        let all_data = api
            .get_all_game_data()
            .await
            .expect("allgamedata should respond in game");
        check_live_game_data(&all_data).expect("allgamedata failed sanity check");

        let stats = api
            .get_live_game_stats()
            .await
            .expect("gamestats should respond in game");
        assert!(
            stats.get("gameMode").is_some() || stats.get("gameTime").is_some(),
            "gamestats should include gameMode or gameTime"
        );

        let players = api
            .get_player_list()
            .await
            .expect("playerlist should respond in game");
        let player_count = players.as_array().map(|a| a.len()).unwrap_or(0);
        assert!(player_count >= 2, "playerlist should have at least 2 players");

        let active_name = api
            .get_active_player_name()
            .await
            .expect("activeplayername should respond in game");
        assert!(
            active_name.as_str().is_some() || active_name.get("summonerName").is_some(),
            "active player name should be present"
        );

        let snapshot = api.get_game_snapshot().await.expect("snapshot");
        assert!(snapshot.live_client_available);
        assert!(
            snapshot.live_game.is_some(),
            "snapshot should include live_game during InProgress"
        );
        assert!(
            snapshot.live_stats.is_some(),
            "snapshot should include live_stats during InProgress"
        );

        if let Some(game_data) = all_data.get("gameData") {
            println!(
                "In-game OK: mode={}, time={}, players={}",
                game_data.get("gameMode").unwrap_or(&serde_json::Value::Null),
                game_data.get("gameTime").unwrap_or(&serde_json::Value::Null),
                player_count
            );
        }
    }

    #[tokio::test]
    async fn test_live_spectator_mode_detection() {
        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = current_phase(&api).await.unwrap_or_default();
        if phase != "InProgress" {
            skip_msg(
                "spectator_detection",
                &format!("current phase is {phase}, need InProgress"),
            );
            return;
        }

        let is_spectator = api
            .detect_spectator_mode()
            .await
            .expect("spectator detection should not error");
        let live_available = api.is_live_client_available().await;

        println!(
            "Spectator check: is_spectator={}, live_client={}",
            is_spectator, live_available
        );

        if is_spectator {
            assert!(
                !live_available,
                "spectator mode should not have live client data"
            );
        } else {
            assert!(
                live_available,
                "non-spectator InProgress should have live client data"
            );
        }
    }
}

#[cfg(test)]
mod capture_fixtures {
    use super::sanity::{check_champion_select_session, check_game_snapshot, check_live_game_data};
    use crate::lcu_api::LCUAPI;
    use crate::league_fixture_capture::{
        capture_enabled, context_from_champion_select, context_from_gameflow_session,
        context_from_live_game, phase_dir, save_fixture, update_capture_manifest, FixtureMeta,
    };
    use std::path::PathBuf;

    async fn connect_live_api() -> Result<LCUAPI, String> {
        let mut api = LCUAPI::new();
        let connection = api.detect_connection().await?;
        api.set_connection(connection);
        Ok(api)
    }

    fn require_capture_enabled() -> bool {
        if capture_enabled() {
            return true;
        }
        println!(
            "[SKIP] fixture capture disabled. Set WINGBOT_CAPTURE_FIXTURES=1 and re-run during the target phase."
        );
        false
    }

    fn skip_wrong_phase(label: &str, actual: &str, expected: &str) -> bool {
        if actual == expected {
            return false;
        }
        println!("[SKIP] {label}: current phase is {actual}, need {expected}");
        true
    }

    #[tokio::test]
    async fn test_capture_champion_select_fixtures() {
        if !require_capture_enabled() {
            return;
        }

        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = api
            .get_current_game_state()
            .await
            .expect("gameflow-phase should respond");
        if skip_wrong_phase("capture_champion_select", &phase, "ChampSelect") {
            return;
        }

        let gameflow = api
            .get_game_flow_session()
            .await
            .expect("gameflow session should respond during ChampSelect");
        let cs = api
            .get_champion_select_state()
            .await
            .expect("champ-select session should respond during ChampSelect");
        check_champion_select_session(&cs).expect("champion select data failed sanity check");

        let mut ctx = context_from_gameflow_session(&gameflow);
        if ctx.slug == "unknown" {
            ctx = context_from_champion_select(&cs);
        }

        let captured_at = chrono::Utc::now().to_rfc3339();
        let gameflow_phase = gameflow
            .get("phase")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let dir = phase_dir(&ctx.slug, "champ_select");

        println!(
            "Capturing champ select fixtures for mode={} queue={:?} -> {}",
            ctx.slug,
            ctx.queue_id,
            dir.display()
        );

        let mut saved: Vec<(String, PathBuf)> = Vec::new();

        let gameflow_path = save_fixture(
            &dir,
            "gameflow_session.json",
            &FixtureMeta {
                captured_at: captured_at.clone(),
                game_mode_slug: ctx.slug.clone(),
                queue_id: ctx.queue_id,
                game_mode: ctx.game_mode.clone(),
                phase: "champ_select".to_string(),
                gameflow_phase: gameflow_phase.clone(),
                source: "lcu/gameflow/session".to_string(),
            },
            &gameflow,
        )
        .expect("save gameflow_session fixture");
        saved.push(("gameflow_session".to_string(), gameflow_path));

        let cs_path = save_fixture(
            &dir,
            "champ_select_session.json",
            &FixtureMeta {
                captured_at: captured_at.clone(),
                game_mode_slug: ctx.slug.clone(),
                queue_id: ctx.queue_id,
                game_mode: ctx.game_mode.clone(),
                phase: "champ_select".to_string(),
                gameflow_phase: gameflow_phase.clone(),
                source: "lcu/champ-select/session".to_string(),
            },
            &cs,
        )
        .expect("save champ_select_session fixture");
        saved.push(("champ_select_session".to_string(), cs_path));

        let snapshot = api
            .get_game_snapshot()
            .await
            .expect("get_game_snapshot should succeed");
        check_game_snapshot(&snapshot).expect("game snapshot failed sanity check");

        let snapshot_value = serde_json::to_value(&snapshot).expect("serialize snapshot");
        let snapshot_path = save_fixture(
            &dir,
            "game_snapshot.json",
            &FixtureMeta {
                captured_at,
                game_mode_slug: ctx.slug.clone(),
                queue_id: ctx.queue_id,
                game_mode: ctx.game_mode,
                phase: "champ_select".to_string(),
                gameflow_phase,
                source: "wingbot/game_snapshot".to_string(),
            },
            &snapshot_value,
        )
        .expect("save game_snapshot fixture");
        saved.push(("game_snapshot".to_string(), snapshot_path));

        update_capture_manifest(&dir, &saved).expect("update champ_select manifest");

        for (name, path) in saved {
            println!("Saved {name}: {}", path.display());
        }
    }

    #[tokio::test]
    async fn test_live_probe_endpoints() {
        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = match api.get_current_game_state().await {
            Ok(p) => format!("OK: {p}"),
            Err(e) => format!("ERR: {e}"),
        };
        let session = match api.get_game_flow_session().await {
            Ok(s) => format!(
                "OK phase={} gameId={:?}",
                s.get("phase").and_then(|v| v.as_str()).unwrap_or("?"),
                s.get("gameData").and_then(|g| g.get("gameId"))
            ),
            Err(e) => format!("ERR: {e}"),
        };
        let live = api.is_live_client_available().await;
        let all_data = match api.get_all_game_data().await {
            Ok(d) => format!(
                "OK mode={} players={}",
                d.get("gameData")
                    .and_then(|g| g.get("gameMode"))
                    .unwrap_or(&serde_json::Value::Null),
                d.get("allPlayers")
                    .and_then(|p| p.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0)
            ),
            Err(e) => format!("ERR: {e}"),
        };
        let snapshot = api.get_game_snapshot().await.expect("snapshot");
        println!("gameflow-phase: {phase}");
        println!("gameflow-session: {session}");
        println!("live_client_available: {live}");
        println!("allgamedata: {all_data}");
        println!(
            "snapshot: phase={} live={} cs={} game={}",
            snapshot.game_phase,
            snapshot.live_client_available,
            snapshot.champion_select.is_some(),
            snapshot.live_game.is_some()
        );
    }

    #[tokio::test]
    async fn test_capture_in_game_fixtures() {
        if !require_capture_enabled() {
            return;
        }

        let api = connect_live_api()
            .await
            .expect("League client must be running");

        let phase = api.resolve_game_phase().await.unwrap_or_else(|e| {
            println!("[SKIP] capture_in_game: cannot read phase: {e}");
            String::new()
        });
        let live_available = api.is_live_client_available().await;
        let in_game_phase = phase == "InProgress" || phase == "GameStart";
        if !in_game_phase && !live_available {
            skip_wrong_phase(
                "capture_in_game",
                if phase.is_empty() { "?" } else { &phase },
                "InProgress or GameStart (or live client)",
            );
            return;
        }
        if !live_available {
            println!(
                "[WARN] live client unavailable during {phase}; capturing LCU session only if available"
            );
        }

        let all_data = if live_available {
            let data = api
                .get_all_game_data()
                .await
                .expect("allgamedata should respond in game");
            check_live_game_data(&data).expect("allgamedata failed sanity check");
            Some(data)
        } else {
            None
        };

        let mut ctx = all_data
            .as_ref()
            .map(context_from_live_game)
            .unwrap_or(crate::league_fixture_capture::GameModeContext {
                slug: "unknown".to_string(),
                queue_id: None,
                game_mode: None,
            });
        if all_data.is_none() {
            if let Ok(session) = api.get_game_flow_session().await {
                ctx = context_from_gameflow_session(&session);
            }
        }
        let captured_at = chrono::Utc::now().to_rfc3339();
        let dir = phase_dir(&ctx.slug, "in_game");

        let gameflow = api.get_game_flow_session().await.ok();
        let gameflow_phase = gameflow
            .as_ref()
            .and_then(|s| s.get("phase"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        println!(
            "Capturing in-game fixtures for mode={} queue={:?} gameMode={:?} -> {}",
            ctx.slug,
            ctx.queue_id,
            ctx.game_mode,
            dir.display()
        );

        let mut saved: Vec<(String, PathBuf)> = Vec::new();
        let base_meta = |source: &str| FixtureMeta {
            captured_at: captured_at.clone(),
            game_mode_slug: ctx.slug.clone(),
            queue_id: ctx.queue_id,
            game_mode: ctx.game_mode.clone(),
            phase: "in_game".to_string(),
            gameflow_phase: gameflow_phase.clone(),
            source: source.to_string(),
        };

        let all_data_path = save_fixture(
            &dir,
            "allgamedata.json",
            &base_meta("live-client/allgamedata"),
            all_data.as_ref().unwrap_or(&serde_json::Value::Null),
        );
        if let Ok(path) = all_data_path {
            saved.push(("allgamedata".to_string(), path));
        }

        if live_available {
            if let Ok(stats) = api.get_live_game_stats().await {
                let stats_path = save_fixture(
                    &dir,
                    "gamestats.json",
                    &base_meta("live-client/gamestats"),
                    &stats,
                )
                .expect("save gamestats fixture");
                saved.push(("gamestats".to_string(), stats_path));
            }

            if let Ok(players) = api.get_player_list().await {
                let players_path = save_fixture(
                    &dir,
                    "playerlist.json",
                    &base_meta("live-client/playerlist"),
                    &players,
                )
                .expect("save playerlist fixture");
                saved.push(("playerlist".to_string(), players_path));
            }

            if let Ok(scores) = api.get_player_scores("").await {
                let scores_path = save_fixture(
                    &dir,
                    "player_scores.json",
                    &base_meta("live-client/playerscores"),
                    &scores,
                )
                .expect("save player_scores fixture");
                saved.push(("player_scores".to_string(), scores_path));
            }

            if let Ok(active_name) = api.get_active_player_name().await {
                let active_path = save_fixture(
                    &dir,
                    "activeplayername.json",
                    &base_meta("live-client/activeplayername"),
                    &active_name,
                )
                .expect("save activeplayername fixture");
                saved.push(("activeplayername".to_string(), active_path));
            }
        }

        if let Some(gameflow) = gameflow {
            let gameflow_path = save_fixture(
                &dir,
                "gameflow_session.json",
                &base_meta("lcu/gameflow/session"),
                &gameflow,
            )
            .expect("save gameflow_session fixture");
            saved.push(("gameflow_session".to_string(), gameflow_path));
        }

        let snapshot = api
            .get_game_snapshot()
            .await
            .expect("get_game_snapshot should succeed");
        if snapshot.live_client_available && !snapshot.is_spectator_mode {
            check_game_snapshot(&snapshot).expect("game snapshot failed sanity check");
        }
        assert!(!saved.is_empty(), "expected at least one fixture to be saved");
        let snapshot_value = serde_json::to_value(&snapshot).expect("serialize snapshot");
        let snapshot_path = save_fixture(
            &dir,
            "game_snapshot.json",
            &base_meta("wingbot/game_snapshot"),
            &snapshot_value,
        )
        .expect("save game_snapshot fixture");
        saved.push(("game_snapshot".to_string(), snapshot_path));

        update_capture_manifest(&dir, &saved).expect("update in_game manifest");

        for (name, path) in saved {
            println!("Saved {name}: {}", path.display());
        }
    }
}

#[cfg(test)]
mod fixture_regression {
    use super::sanity::{check_champion_select_session, check_live_game_data};
    use crate::league_fixture_capture::{fixtures_root, list_fixture_files, load_fixture_data};

    #[test]
    fn test_regression_captured_champion_select_fixtures() {
        let files = list_fixture_files("champ_select_session.json");
        if files.is_empty() {
            println!(
                "[SKIP] no captured champ_select_session.json fixtures under {}",
                fixtures_root().display()
            );
            return;
        }

        for relative in files {
            let path = fixtures_root().join(&relative);
            let data = load_fixture_data(&path).expect("load champ select fixture");
            check_champion_select_session(&data)
                .unwrap_or_else(|e| panic!("{} failed sanity check: {e}", path.display()));
            println!("Validated champ select fixture: {}", path.display());
        }
    }

    #[test]
    fn test_regression_captured_in_game_fixtures() {
        let files = list_fixture_files("allgamedata.json");
        if files.is_empty() {
            println!(
                "[SKIP] no captured allgamedata.json fixtures under {}",
                fixtures_root().display()
            );
            return;
        }

        for relative in files {
            let path = fixtures_root().join(&relative);
            let data = load_fixture_data(&path).expect("load in-game fixture");
            check_live_game_data(&data)
                .unwrap_or_else(|e| panic!("{} failed sanity check: {e}", path.display()));
            println!("Validated in-game fixture: {}", path.display());
        }
    }
}

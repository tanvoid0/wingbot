#!/usr/bin/env node
/**
 * WingBot LLM pipeline — local LoL knowledge prep, remote train/export via agent-platform.
 *
 * Usage:
 *   node llm/scripts/run-pipeline.mjs setup
 *   node llm/scripts/run-pipeline.mjs prepare --project wingbot-lol
 *   node llm/scripts/run-pipeline.mjs sync --project wingbot-lol
 *   node llm/scripts/run-pipeline.mjs train --project wingbot-lol
 *   node llm/scripts/run-pipeline.mjs all --project wingbot-lol --train
 */

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runRemotePipeline, syncProject, startBuildJob, pollJob } from "./agent-platform-client.mjs";
import { dockerExport, dockerTrain, trainContainerRunning } from "./docker-train.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const LLM_ROOT = path.resolve(__dirname, "..");
const CORE = path.join(LLM_ROOT, "core");
const isWin = process.platform === "win32";

function parseArgs(argv) {
  const positional = [];
  const flags = new Set();
  const options = { project: "wingbot-lol", players: null, matchesPerPlayer: null, tier: null };

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--train") flags.add("train");
    else if (arg === "--skip-ddragon") flags.add("skip-ddragon");
    else if (arg === "--ddragon") flags.add("ddragon");
    else if (arg === "--no-sync") flags.add("no-sync");
    else if (arg === "--offline") flags.add("offline");
    else if (arg === "--offline-eval") flags.add("offline-eval");
    else if (arg === "--require-matchup") flags.add("require-matchup");
    else if (arg === "--skip-ingest") flags.add("skip-ingest");
    else if (arg === "--with-timeline") flags.add("with-timeline");
    else if (arg === "--force") flags.add("force");
    else if (arg === "--project") options.project = argv[++i] ?? options.project;
    else if (arg === "--players") options.players = Number(argv[++i]);
    else if (arg === "--matches-per-player") options.matchesPerPlayer = Number(argv[++i]);
    else if (arg === "--tier") options.tier = argv[++i];
    else if (!arg.startsWith("-")) positional.push(arg);
  }

  return { command: positional[0] ?? "all", flags, options };
}

function venvPython() {
  const bin = isWin
    ? path.join(CORE, ".venv", "Scripts", "python.exe")
    : path.join(CORE, ".venv", "bin", "python");
  return bin;
}

function venvPip() {
  const bin = isWin
    ? path.join(CORE, ".venv", "Scripts", "pip.exe")
    : path.join(CORE, ".venv", "bin", "pip");
  return bin;
}

function run(cmd, args, opts = {}) {
  const result = spawnSync(cmd, args, {
    cwd: opts.cwd ?? CORE,
    stdio: "inherit",
    shell: false,
    env: process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function runPython(script, args = [], opts = {}) {
  run(venvPython(), [script, ...args], opts);
}

function ensureVenv() {
  const py = venvPython();
  if (!existsSync(py)) {
    console.log("Creating Python venv in llm/core/.venv ...");
    run("python", ["-m", "venv", path.join(CORE, ".venv")], { cwd: CORE });
  }
}

function setup() {
  ensureVenv();
  console.log("Installing ingest dependencies (no GPU torch) ...");
  run(venvPip(), ["install", "-r", path.join(CORE, "requirements.txt")]);
  console.log("");
  console.log("Training runs on agent-platform (Docker train worker or remote API).");
  console.log("Set AGENT_PLATFORM_MASTER_KEY and AGENT_PLATFORM_BASE_URL=http://127.0.0.1:18410");
}

function fetchRiot(project, opts = {}) {
  ensureVenv();
  const args = ["--project", project];
  if (opts.tier) args.push("--tier", opts.tier);
  if (opts.players) args.push("--players", String(opts.players));
  if (opts.matchesPerPlayer) args.push("--matches-per-player", String(opts.matchesPerPlayer));
  if (opts.platform) args.push("--platform", opts.platform);
  if (opts.withTimeline) args.push("--with-timeline");
  if (opts.force) args.push("--force");
  runPython(path.join(CORE, "fetch_riot_matches.py"), args);
}

function ingestExternal(project, opts = {}) {
  ensureVenv();
  const args = ["--project", project];
  if (opts.limit) args.push("--limit", String(opts.limit));
  if (opts.maxExamples) args.push("--max-examples", String(opts.maxExamples));
  if (opts.requireMatchup) args.push("--require-matchup");
  if (opts.source) args.push("--source", opts.source);
  runPython(path.join(CORE, "ingest_external.py"), args);
}

function prepare(project, skipDdragon, ingestOpts = {}) {
  ensureVenv();
  const projScripts = path.join(LLM_ROOT, "projects", project, "scripts");

  if (!skipDdragon) {
    runPython(path.join(projScripts, "extract_ddragon.py"), [], { cwd: projScripts });
  }
  runPython(path.join(projScripts, "export_wingbot_rules.py"), [], { cwd: projScripts });
  runPython(path.join(projScripts, "generate_matchups.py"), [], { cwd: projScripts });
  runPython(path.join(CORE, "merge_knowledge.py"), ["--project", project]);
  if (ingestOpts.enabled !== false) {
    ingestExternal(project, { source: "all", ...ingestOpts });
  }
  runPython(path.join(CORE, "build_dataset.py"), ["--project", project]);
}

async function sync(project, flags) {
  await syncProject(project, { includeDdragon: flags.has("ddragon") });
}

async function trainRemote(project, flags) {
  await syncProject(project, { includeDdragon: flags.has("ddragon") });

  if (trainContainerRunning()) {
    dockerTrain(project);
    dockerExport(project);
    const job = await startBuildJob(project, ["eval"], { offlineEval: flags.has("offline-eval") });
    const final = await pollJob(job.id);
    if (final.status !== "succeeded") process.exit(1);
    return;
  }

  console.warn("Train container not running — falling back to API job (needs GPU deps on server).");
  await runRemotePipeline(project, ["train", "export", "eval"], {
    sync: false,
    includeDdragon: flags.has("ddragon"),
    offlineEval: flags.has("offline-eval"),
  });
}

async function evalRemote(project, flags) {
  await runRemotePipeline(project, ["eval"], {
    sync: !flags.has("no-sync"),
    offlineEval: flags.has("offline") || flags.has("offline-eval"),
  });
}

async function all({ project, flags }) {
  prepare(project, flags.has("skip-ddragon"));
  await sync(project, flags);

  if (flags.has("train")) {
    if (trainContainerRunning()) {
      dockerTrain(project);
      dockerExport(project);
      const job = await startBuildJob(project, ["eval"], { offlineEval: false });
      await pollJob(job.id);
    } else {
      await runRemotePipeline(project, ["train", "export", "eval"], {
        sync: false,
        offlineEval: false,
      });
    }
  } else {
    await runRemotePipeline(project, ["eval"], {
      sync: false,
      offlineEval: true,
    });
  }

  console.log(`\nPipeline complete for ${project}`);
}

function printHelp() {
  console.log(`WingBot LLM pipeline (local LoL prep → agent-platform train/export)

Commands:
  setup       Create venv and install ingest-only Python deps
  prepare     LoL knowledge extract, merge, ingest, build datasets (local)
  sync        Upload project artifacts to agent-platform model-ops
  ingest      External match data only
  fetch-riot  Bulk-fetch ranked matches (needs RIOT_API_KEY)
  train       Remote: sync + train + export + eval on agent-platform
  eval        Remote eval job on agent-platform
  all         prepare + sync + offline eval (+ train/export/live eval with --train)

Options:
  --project <name>     Project id (default: wingbot-lol)
  --train              Include remote train + export in "all"
  --skip-ddragon       Skip Data Dragon download in prepare
  --ddragon            Include ddragon knowledge in sync (large)
  --skip-ingest        Skip HuggingFace ingest in prepare
  --no-sync            Skip upload before remote job
  --offline-eval       Eval without Ollama
  --require-matchup    Ingest only curated matchup rows

Env:
  AGENT_PLATFORM_BASE_URL      default http://127.0.0.1:18410
  AGENT_PLATFORM_MASTER_KEY    Bearer token for model-ops API
`);
}

const { command, flags, options } = parseArgs(process.argv.slice(2));

async function main() {
  switch (command) {
    case "setup":
      setup();
      break;
    case "prepare":
      prepare(options.project, flags.has("skip-ddragon"), {
        enabled: !flags.has("skip-ingest"),
      });
      break;
    case "sync":
      await sync(options.project, flags);
      break;
    case "ingest":
      ingestExternal(options.project, {
        source: process.env.LLM_INGEST_SOURCE || "all",
        requireMatchup: flags.has("require-matchup"),
      });
      break;
    case "fetch-riot":
      fetchRiot(options.project, {
        tier: options.tier || process.env.LLM_RIOT_TIER || "challenger",
        players: options.players || (process.env.LLM_RIOT_PLAYERS ? Number(process.env.LLM_RIOT_PLAYERS) : 30),
        matchesPerPlayer:
          options.matchesPerPlayer || (process.env.LLM_RIOT_MATCHES ? Number(process.env.LLM_RIOT_MATCHES) : 15),
        withTimeline: flags.has("with-timeline"),
        force: flags.has("force"),
      });
      ingestExternal(options.project, { source: "riot-matches" });
      break;
    case "train":
      await trainRemote(options.project, flags);
      break;
    case "export":
      console.log("Export is part of remote train; running train + export + eval ...");
      await trainRemote(options.project, flags);
      break;
    case "eval":
      await evalRemote(options.project, flags);
      break;
    case "all":
      await all({ project: options.project, flags });
      break;
    case "help":
    case "--help":
    case "-h":
      printHelp();
      break;
    default:
      console.error(`Unknown command: ${command}`);
      printHelp();
      process.exit(1);
  }
}

main().catch((err) => {
  console.error(err.message || err);
  process.exit(1);
});

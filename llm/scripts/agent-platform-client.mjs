#!/usr/bin/env node
/**
 * WingBot → Agent Platform model-ops client.
 *
 * Env:
 *   AGENT_PLATFORM_BASE_URL  default http://127.0.0.1:18410
 *   AGENT_PLATFORM_MASTER_KEY or AGENT_PLATFORM_TOKEN
 */

import { createReadStream, existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const LLM_ROOT = path.resolve(__dirname, "..");

export function platformConfig() {
  const base = (process.env.AGENT_PLATFORM_BASE_URL || "http://127.0.0.1:18410").replace(/\/$/, "");
  let token = process.env.AGENT_PLATFORM_MASTER_KEY || process.env.AGENT_PLATFORM_TOKEN;
  if (token) token = token.replace(/^["']|["']$/g, "");
  if (!token) {
    throw new Error("Set AGENT_PLATFORM_MASTER_KEY or AGENT_PLATFORM_TOKEN");
  }
  return { base, token };
}

export async function apiFetch(pathname, { method = "GET", body, headers = {} } = {}) {
  const { base, token } = platformConfig();
  const url = `${base}${pathname.startsWith("/") ? pathname : `/${pathname}`}`;
  const res = await fetch(url, {
    method,
    body,
    headers: {
      Authorization: `Bearer ${token}`,
      ...headers,
    },
  });
  const text = await res.text();
  let data;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    data = text;
  }
  if (!res.ok) {
    const detail = typeof data === "object" && data && "detail" in data ? data.detail : text;
    throw new Error(`${method} ${pathname} → ${res.status}: ${detail}`);
  }
  return data;
}

export async function ensureProject(project, manifest = {}) {
  try {
    return await apiFetch(`/api/v1/model-ops/projects/${encodeURIComponent(project)}`);
  } catch (err) {
    if (!String(err.message).includes("404")) throw err;
  }
  return apiFetch("/api/v1/model-ops/projects", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      name: project,
      description: manifest.description || `WingBot project ${project}`,
      base_model: manifest.base_model,
      ollama_tag: manifest.ollama_tag || project,
    }),
  });
}

function loadProjectManifest(project) {
  const yamlPath = path.join(LLM_ROOT, "projects", project, "project.yaml");
  if (!existsSync(yamlPath)) {
    throw new Error(`Missing ${yamlPath}`);
  }
  const raw = readFileSync(yamlPath, "utf8");
  const manifest = {};
  for (const line of raw.split("\n")) {
    const m = line.match(/^([a-z_]+):\s*(.+)$/);
    if (!m) continue;
    const [, key, val] = m;
    manifest[key] = val.replace(/^["']|["']$/g, "");
  }
  return manifest;
}

function collectFiles(rootDir, relPrefix = "") {
  const out = [];
  if (!existsSync(rootDir)) return out;
  for (const name of readdirSync(rootDir)) {
    const abs = path.join(rootDir, name);
    const rel = relPrefix ? `${relPrefix}/${name}` : name;
    const st = statSync(abs);
    if (st.isDirectory()) {
      out.push(...collectFiles(abs, rel));
    } else if (st.isFile()) {
      out.push({ rel: rel.replace(/\\/g, "/"), abs });
    }
  }
  return out;
}

async function uploadProjectFiles(project, fileEntries) {
  if (fileEntries.length === 0) return 0;
  const { base, token } = platformConfig();
  const form = new FormData();
  for (const { rel, abs } of fileEntries) {
    const blob = new Blob([readFileSync(abs)]);
    form.append("files", blob, rel);
  }
  const res = await fetch(`${base}/api/v1/model-ops/projects/${encodeURIComponent(project)}/files`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: form,
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(`upload files → ${res.status}: ${JSON.stringify(data)}`);
  }
  return data.uploaded ?? fileEntries.length;
}

export async function syncProject(project, opts = {}) {
  const includeDdragon = opts.includeDdragon === true;
  const manifest = loadProjectManifest(project);
  await ensureProject(project, manifest);

  const projectDir = path.join(LLM_ROOT, "projects", project);
  const uploads = [];

  const singles = [
    "project.yaml",
    "schemas/game_state.schema.json",
    "export/system.txt",
    "export/Modelfile",
    "datasets/train.jsonl",
    "datasets/eval.jsonl",
  ];
  for (const rel of singles) {
    const abs = path.join(projectDir, rel);
    if (existsSync(abs)) uploads.push({ rel, abs });
  }

  const externalDir = path.join(projectDir, "datasets", "external");
  for (const { rel, abs } of collectFiles(externalDir, "datasets/external")) {
    if (rel.endsWith(".jsonl")) uploads.push({ rel, abs });
  }

  const knowledgeDirs = ["knowledge/rules", "knowledge/matchups", "knowledge/decision_patterns.jsonl"];
  if (includeDdragon) knowledgeDirs.unshift("knowledge/ddragon");
  for (const relDir of knowledgeDirs) {
    const absDir = path.join(projectDir, relDir);
    if (existsSync(absDir) && statSync(absDir).isFile()) {
      uploads.push({ rel: relDir, abs: absDir });
      continue;
    }
    for (const { rel, abs } of collectFiles(absDir, relDir)) {
      uploads.push({ rel, abs });
    }
  }

  const shared = path.join(LLM_ROOT, "projects", "_shared", "decision_patterns.jsonl");
  if (existsSync(shared)) {
    uploads.push({ rel: "knowledge/_shared/decision_patterns.jsonl", abs: shared });
  }

  console.log(`Uploading ${uploads.length} files to agent-platform project "${project}" ...`);
  const uploaded = await uploadProjectFiles(project, uploads);
  console.log(`Uploaded ${uploaded} files.`);
  return { uploaded, manifest };
}

export async function startBuildJob(project, stages, opts = {}) {
  const body = {
    project,
    stages,
    offline_eval: opts.offlineEval ?? false,
    register_alias: opts.registerAlias ?? null,
  };
  const job = await apiFetch("/api/v1/model-ops/jobs", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  console.log(`Started job #${job.id} (${job.job_type}) stages=${stages.join(",")}`);
  console.log(`  poll: ${job.poll_url}`);
  return job;
}

export async function pollJob(jobId, { intervalMs = 2000, onUpdate } = {}) {
  let job = await apiFetch(`/api/v1/model-ops/jobs/${jobId}`);
  while (job.status === "pending" || job.status === "running") {
    if (onUpdate) onUpdate(job);
    await new Promise((r) => setTimeout(r, intervalMs));
    job = await apiFetch(`/api/v1/model-ops/jobs/${jobId}`);
  }
  if (onUpdate) onUpdate(job);
  return job;
}

export async function runRemotePipeline(project, stages, opts = {}) {
  if (opts.sync !== false) {
    await syncProject(project, { includeDdragon: opts.includeDdragon });
  }
  const job = await startBuildJob(project, stages, opts);
  if (opts.noWait) return job;
  const final = await pollJob(job.id, {
    onUpdate(j) {
      const tail = (j.log_tail || "").split("\n").slice(-3).join("\n");
      process.stdout.write(`\r[${j.status}] ${j.current_stage || "-"}`.padEnd(40));
      if (tail) process.stdout.write(` | ${tail.slice(-80)}`);
    },
  });
  console.log("\n");
  console.log(`Job #${final.id} finished: ${final.status}`);
  if (final.error_message) console.error(final.error_message);
  if (final.status !== "succeeded") process.exit(1);
  return final;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [cmd, project = "wingbot-lol", ...rest] = process.argv.slice(2);
  const flags = new Set(rest.filter((a) => a.startsWith("--")));
  const stagesArg = rest.find((a) => !a.startsWith("--"));
  try {
    if (cmd === "sync") {
      await syncProject(project, { includeDdragon: flags.has("--ddragon") });
    } else if (cmd === "job") {
      const stages = (stagesArg || "train,export,eval").split(",").map((s) => s.trim());
      await runRemotePipeline(project, stages, {
        sync: !flags.has("--no-sync"),
        includeDdragon: flags.has("--ddragon"),
        offlineEval: flags.has("--offline-eval"),
        noWait: flags.has("--no-wait"),
      });
    } else {
      console.log(`Usage:
  node agent-platform-client.mjs sync [project] [--ddragon]
  node agent-platform-client.mjs job [project] [stages] [--no-sync] [--offline-eval] [--no-wait]
`);
      process.exit(1);
    }
  } catch (err) {
    console.error(err.message || err);
    process.exit(1);
  }
}

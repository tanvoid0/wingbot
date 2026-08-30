#!/usr/bin/env node
/**
 * Run train + export inside agent-platform Docker train worker (shared MODEL_OPS_DATA_DIR).
 */

import { spawnSync } from "node:child_process";

const TRAIN_CONTAINER = process.env.AGENT_PLATFORM_TRAIN_CONTAINER || "agent-platform-train-1";

const REGISTRY_BOOT = `
import os
os.environ.setdefault("MODEL_OPS_DATA_DIR", "/app/data/llm/model_ops")
from model_ops.service import wire_registry_callback
wire_registry_callback(None)
`;

export function trainContainerRunning(name = TRAIN_CONTAINER) {
  const r = spawnSync("docker", ["inspect", "-f", "{{.State.Running}}", name], { encoding: "utf8" });
  return r.status === 0 && r.stdout.trim() === "true";
}

function dockerPython(body, project) {
  const script = `${REGISTRY_BOOT}\n${body.replaceAll("${project}", project)}`;
  const r = spawnSync(
    "docker",
    ["exec", "-e", "MODEL_OPS_GPU_SUBPROCESS=0", "-e", "MODEL_OPS_DATA_DIR=/app/data/llm/model_ops", TRAIN_CONTAINER, "python", "-c", script],
    { stdio: "inherit" },
  );
  if (r.status !== 0) {
    throw new Error(`docker exec train failed (exit ${r.status ?? 1})`);
  }
}

export function dockerTrain(project) {
  if (!trainContainerRunning()) {
    throw new Error(
      `Train container "${TRAIN_CONTAINER}" is not running. Start with:\n` +
        "  docker compose -f docker-compose.yml -f docker-compose.train.yml --profile train up -d train",
    );
  }
  console.log(`Training ${project} in ${TRAIN_CONTAINER} ...`);
  dockerPython("from model_ops.pipeline.train_lora import train\ntrain('${project}')", project);
}

export function dockerExport(project) {
  dockerPython(
    "from model_ops.pipeline.export_ollama import merge_and_export_gguf\nmerge_and_export_gguf('${project}')",
    project,
  );
}

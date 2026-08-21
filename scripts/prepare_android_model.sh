#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLAMA_DIR="$ROOT/android/app/src/main/cpp/llama.cpp"
MODEL_DIR="$ROOT/android/app/src/main/assets/models"
LLAMA_COMMIT="9a286ac98d2cab74231bd3f1fc3f2b8bdf05422e"
MODEL_URL="https://huggingface.co/tensorblock/SmolLM2-135M-Instruct-GGUF/resolve/main/SmolLM2-135M-Instruct-Q2_K.gguf?download=true"
MODEL_FILE="SmolLM2-135M-Instruct-Q2_K.gguf"

mkdir -p "$(dirname "$LLAMA_DIR")" "$MODEL_DIR"

if [[ ! -d "$LLAMA_DIR/.git" ]]; then
  git clone https://github.com/ggml-org/llama.cpp.git "$LLAMA_DIR"
fi

git -C "$LLAMA_DIR" fetch --depth 1 origin "$LLAMA_COMMIT"
git -C "$LLAMA_DIR" checkout "$LLAMA_COMMIT"

if [[ ! -s "$MODEL_DIR/$MODEL_FILE" ]]; then
  curl -L --fail --retry 3 "$MODEL_URL" -o "$MODEL_DIR/$MODEL_FILE"
fi

sha256sum "$MODEL_DIR/$MODEL_FILE" | tee "$MODEL_DIR/$MODEL_FILE.sha256"

echo "Prepared pinned llama.cpp and $MODEL_FILE"

#!/usr/bin/env bash
set -euo pipefail

SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="$HOME/.local/bin"
CMD_NAME="cx"

usage() {
  echo "Usage: $(basename "$0") [-n|--name <command>] (default: cx)"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -n|--name)
      shift
      if [[ $# -eq 0 ]]; then
        echo "Error: --name requires a value"
        usage
        exit 1
      fi
      CMD_NAME="$1"
      shift
      ;;
    --name=*)
      CMD_NAME="${1#*=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Error: unknown option $1"
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$CMD_NAME" ]]; then
  echo "Error: command name cannot be empty"
  exit 1
fi

if [[ "$CMD_NAME" == */* ]]; then
  echo "Error: command name must not contain '/'"
  exit 1
fi

TARGET="$BIN_DIR/$CMD_NAME"

mkdir -p "$BIN_DIR"

if [[ -L "$TARGET" || -e "$TARGET" ]]; then
  if [[ "$(readlink "$TARGET")" == "$SRC_DIR/cx" ]]; then
    echo "$CMD_NAME already installed"
    exit 0
  fi
  echo "Error: $TARGET exists and is not cx symlink"
  exit 1
fi

ln -s "$SRC_DIR/cx" "$TARGET"
chmod +x "$SRC_DIR/cx"

echo "Installed cx as $CMD_NAME → $TARGET"

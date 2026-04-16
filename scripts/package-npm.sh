#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
artifacts_dir="${2:-dist/artifacts}"
out_dir="${3:-dist/npm}"

if [[ -z "${version}" ]]; then
  echo "Usage: $0 <version> [artifacts_dir] [out_dir]" >&2
  exit 1
fi

version="${version#v}"

if [[ ! -d "${artifacts_dir}" ]]; then
  echo "Missing artifacts dir: ${artifacts_dir}" >&2
  exit 1
fi

root_package_name="$(node -p "require('./package.json').name")"
repository_url="$(node -p "const repo = require('./package.json').repository; typeof repo === 'string' ? repo : repo.url")"

rm -rf "${out_dir}"
mkdir -p "${out_dir}"

shopt -s nullglob
artifact_dirs=("${artifacts_dir}"/codex-profiles-*)
if [[ ${#artifact_dirs[@]} -eq 0 ]]; then
  echo "No build artifacts found under ${artifacts_dir}" >&2
  exit 1
fi

for artifact_dir in "${artifact_dirs[@]}"; do
  target="${artifact_dir##*/codex-profiles-}"
  pkg_suffix=""
  os=""
  cpu=""
  bin_name="codex-profiles"

  case "${target}" in
    x86_64-unknown-linux-gnu)
      pkg_suffix="linux-x64"
      os="linux"
      cpu="x64"
      ;;
    aarch64-unknown-linux-gnu)
      pkg_suffix="linux-arm64"
      os="linux"
      cpu="arm64"
      ;;
    x86_64-apple-darwin)
      pkg_suffix="darwin-x64"
      os="darwin"
      cpu="x64"
      ;;
    aarch64-apple-darwin)
      pkg_suffix="darwin-arm64"
      os="darwin"
      cpu="arm64"
      ;;
    x86_64-pc-windows-msvc)
      pkg_suffix="win32-x64"
      os="win32"
      cpu="x64"
      bin_name="codex-profiles.exe"
      ;;
    *)
      echo "Skipping unsupported target ${target}" >&2
      continue
      ;;
  esac

  pkg="${root_package_name}-${pkg_suffix}"

  pkg_dir="${out_dir}/${pkg}"
  mkdir -p "${pkg_dir}/bin"
  if [[ ! -f "${artifact_dir}/${bin_name}" ]]; then
    echo "Missing binary for ${target}: ${artifact_dir}/${bin_name}" >&2
    exit 1
  fi
  cp "${artifact_dir}/${bin_name}" "${pkg_dir}/bin/${bin_name}"
  if [[ "${bin_name}" != *".exe" ]]; then
    chmod +x "${pkg_dir}/bin/${bin_name}"
  fi

  cat > "${pkg_dir}/package.json" <<JSON
{
  "name": "${pkg}",
  "version": "${version}",
  "license": "MIT",
  "publishConfig": {
    "access": "public"
  },
  "os": ["${os}"],
  "cpu": ["${cpu}"],
  "files": ["bin"],
  "description": "Prebuilt native binary for Codex Profiles",
  "repository": {
    "type": "git",
    "url": "${repository_url}"
  }
}
JSON
done
shopt -u nullglob

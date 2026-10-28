#!/usr/bin/env bash
# YT-096: fresh TUF authentication, KMS bootstrap, then upstream keyless verification.
# Public inputs only. This script never requests a Yetlet identity or signs a payload.
set -euo pipefail
: "${BOOTSTRAP:?set an absolute empty task-scoped directory}"
[[ "$BOOTSTRAP" == /* && -d "$BOOTSTRAP" && ! -L "$BOOTSTRAP" ]]
[[ -z "$(find "$BOOTSTRAP" -mindepth 1 -print -quit)" ]]
sign_scripts="$(cd -- "$(dirname -- "$0")" && pwd)"
umask 077
mkdir -m 700 "$BOOTSTRAP/age-tools"
BOOTSTRAP="$BOOTSTRAP/age-tools" bash "$sign_scripts/bootstrap-release-age.sh"
python3 -m venv --without-pip "$BOOTSTRAP/venv"
curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 --max-time 120 \
  --output "$BOOTSTRAP/pip.whl" \
  https://files.pythonhosted.org/packages/f3/6e/1736e5b4ae2b778ef2f81c47d797de9f891d4d8acb047a24ca37a60294dd/pip-26.2.1-py3-none-any.whl
printf '%s  %s\n' 71138adf1f4ca900cdb7d289c21b7494329f2332b6d85f0e1c42108c0384ed3e "$BOOTSTRAP/pip.whl" | sha256sum --check --strict
PYTHONPATH="$BOOTSTRAP/pip.whl" "$BOOTSTRAP/venv/bin/python" -m pip --isolated install \
  --require-hashes --only-binary=:all: --index-url https://pypi.org/simple \
  --requirement "$sign_scripts/release-sign-tools.txt"
"$BOOTSTRAP/venv/bin/python" "$sign_scripts/release-sign-bootstrap.py" "$BOOTSTRAP"

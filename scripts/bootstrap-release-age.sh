#!/usr/bin/env bash
# YT-096: authenticated Linux amd64 encryption tool; no private identity.
set -euo pipefail
: "${BOOTSTRAP:?set an absolute empty task-scoped directory}"
[[ "$BOOTSTRAP" == /* && -d "$BOOTSTRAP" && -z "$(find "$BOOTSTRAP" -mindepth 1 -print -quit)" ]]
cd "$BOOTSTRAP"
curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
  --max-time 120 --output go1.27.3.linux-amd64.tar.gz \
  https://go.dev/dl/go1.27.3.linux-amd64.tar.gz
printf '%s\n' '65efbc80fdc1fa2cd2631cad5b481777a68e8725f34e89f50fc6ac714e379b40  go1.27.3.linux-amd64.tar.gz' | sha256sum --check --strict
tar --extract --gzip --file go1.27.3.linux-amd64.tar.gz --no-same-owner
export GOTOOLCHAIN=local GOENV=off GOSUMDB=sum.golang.org
export GOPROXY=https://proxy.golang.org CGO_ENABLED=0
# Keep this task-private module cache removable by ordinary owned-work cleanup.
export GOPRIVATE= GONOSUMDB= GONOPROXY= GOFLAGS=-modcacherw GOWORK=off
export GOBIN="$BOOTSTRAP/bin" GOPATH="$BOOTSTRAP/gopath"
export GOMODCACHE="$BOOTSTRAP/gomodcache" GOCACHE="$BOOTSTRAP/gocache"
"$BOOTSTRAP/go/bin/go" version
"$BOOTSTRAP/go/bin/go" install -trimpath sigsum.org/sigsum-go/cmd/sigsum-verify@v0.13.1
printf '%s\n' \
  '747d88da11131cd3e26841965e8e3b95057a6cfabda993dc0f43cd913512479a  gomodcache/cache/download/sigsum.org/sigsum-go/@v/v0.13.1.zip' \
  '339c97f9481d2cb3c6011491a8fa576a85727702baf1d351976e26a6e7ff0471  gomodcache/cache/download/sigsum.org/sigsum-go/@v/v0.13.1.mod' | sha256sum --check --strict
# The checksum-authenticated module release maps to this reviewed source commit.
readonly sigsum_source_commit=c537b12e18bdc787b5ed300eea630f3cb4e988bf
grep -F "$sigsum_source_commit" "$GOMODCACHE/cache/download/sigsum.org/sigsum-go/@v/v0.13.1.info"
"$BOOTSTRAP/go/bin/go" version -m "$GOBIN/sigsum-verify" > verifier-build.txt
grep -F 'h1:q3LCLow8dWmy2hivXhEwu2uNeCwigIbLMwS1+CD35Lo=' verifier-build.txt
"$BOOTSTRAP/go/bin/go" mod download -json sigsum.org/sigsum-go@v0.13.1 > verifier-module.json
grep -F 'h1:7ID0cqoLWQ+9B8bGnVdp58/Hwdfj132XDgU6CP6W2ck=' verifier-module.json
cp "$GOMODCACHE/sigsum.org/sigsum-go@v0.13.1/pkg/policy/builtin/sigsum-generic-2025-1.builtin-policy" sigsum-generic-2025-1.policy
cat >age-sigsum-key.pub <<'KEYS'
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIM1WpnEswJLPzvXJDiswowy48U+G+G1kmgwUE2eaRHZG
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAz2WM5CyPLqiNjk7CLl4roDXwKhQ0QExXLebukZEZFS
KEYS
printf '%s\n' \
  '666d9d0b9ab2e4019769c42eaccd7d6d502a9abac45979bb9b08b7213e4f53e3  sigsum-generic-2025-1.policy' \
  'ca60c0504ac65cc43ef180a34579ce6ed8f909b9136afef0ca42c88280f42cdc  age-sigsum-key.pub' | sha256sum --check --strict
curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
  --max-time 120 --output age-v1.3.2-linux-amd64.tar.gz \
  https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz
curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
  --max-time 120 --output age-v1.3.2-linux-amd64.tar.gz.proof \
  https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz.proof
printf '%s\n' \
  'cbe24006683f8eb669266162894b9a522a1af52f2665fbc63a4bb032ed26ac10  age-v1.3.2-linux-amd64.tar.gz' \
  '98402fa59b4e433426156b4a7e74a32e44902e0434c61158f7de18ffc84f1f97  age-v1.3.2-linux-amd64.tar.gz.proof' | sha256sum --check --strict
"$GOBIN/sigsum-verify" -k age-sigsum-key.pub \
  -p "$BOOTSTRAP/sigsum-generic-2025-1.policy" \
  age-v1.3.2-linux-amd64.tar.gz.proof <age-v1.3.2-linux-amd64.tar.gz
tar --extract --gzip --file age-v1.3.2-linux-amd64.tar.gz --no-same-owner
"$BOOTSTRAP/age/age" --version
printf '%s\n' 'eb7dd1b518f0a307c99cd97782623c5321da049154b04acd2d98d21aa7bc9b2c  age/age' | sha256sum --check --strict

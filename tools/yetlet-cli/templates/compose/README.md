# Generated Compose configuration (native-ssr-v1)

This directory contains exactly `compose.yaml` and this README. Generation only
reads the selected app's explicit lock, unchanged cloud profile and supported
framework closure; it invokes no Cargo, shell, Docker, registry or network client.
No source paths, environment values, credentials or source exports are included.

The `app` service references the exact operator-selected image. Build and audit
that image separately using the YT-089b Docker generator's three sanitized source
contexts and locked build instructions. The existing image contract is Linux
OCI/amd64 with GNU/glibc, a non-root exec entrypoint and app-owned SIGTERM handling.
Windows hosts generate the same Linux-targeted configuration. No image is built,
pulled, published or attested by this generator. `pull_policy: never` requires the
selected image to exist locally; for a digest reference it must be locally
resolvable by that exact digest. Loading a tag alone may not provide that identity.

Image syntax is deliberately bounded: lowercase ASCII registry/repository and
either an explicit non-`latest` tag or `@sha256:` followed by 64 lowercase hex
digits. Registries must be dotted DNS/IPv4 hosts or `localhost`, optionally with
a decimal port 1-65535 without leading zeros. No short/unqualified names, IPv6,
tag-plus-digest, escapes, Unicode or implicit/default tag are supported. YAML
scalars are single quoted; the validated image grammar excludes quotes, dollar
signs, backslashes and controls. The image is never inserted into a shell command.

Container settings are explicitly `YETLET_HOST=0.0.0.0` and `YETLET_PORT=8080`;
only host `127.0.0.1:8080` is published. No host networking, volumes, secrets,
privileged mode, build context or implicit environment-file substitution is used.
Compose itself can read a project `.env` when separately invoked; this YAML has
no interpolation or `env_file`, and the Yetlet generator never reads `.env`.
An operator must choose a protected project directory and explicit Compose file.

`stop_grace_period: 30s` is a conservative example operator budget, after which
Compose can force termination. The accepted cloud app initiates graceful drain
on SIGTERM and stops readiness, but has **no application drain deadline**.
Long requests/streams may exceed 30 seconds; measure them and adjust this value
and traffic withdrawal before operational use. This setting is no guarantee of
successful drain or production readiness.

No HEALTHCHECK is generated: the minimal image does not promise curl/wget or a
probe helper. The app's `/health/live` and `/health/ready` endpoints report local
process state only. Check HTTP/CSP and real SIGTERM/drain separately; a running
container or syntactically valid Compose model is not a health claim.

The following are separate, explicit **operator actions**, never CLI side effects.
Use an isolated project name and a disposable local image in an authorized lab;
ensure port 8080 is free and retain the same project name for cleanup:

```sh
docker compose -p yetlet-compose-lab -f compose.yaml config
docker compose -p yetlet-compose-lab -f compose.yaml up -d --no-build --pull never
docker compose -p yetlet-compose-lab -f compose.yaml down --timeout 30
```

These fixed commands also work in PowerShell. They do not interpolate the image.
Runtime Compose validation and container actions require separate authorization
and a compatible installed Compose version. This output alone proves neither.

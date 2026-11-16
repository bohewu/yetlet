
## Service profile (`--service`)

This section applies only when this application was created with `yetlet new --service`.
The Service profile exposes two application-owned health routes:

- `GET /health/live` returns HTTP 200 while the process is serving. It is a liveness
  signal only; it does not check databases, queues or other dependencies.
- `GET /health/ready` returns HTTP 200 while the local process is ready and changes
  to HTTP 503 after a handled shutdown signal, before Axum begins graceful shutdown.

The Service profile handles SIGINT/SIGTERM on Unix and Ctrl+C/Ctrl+Break on Windows.
It marks the process unready before graceful server shutdown. This is a local
process lifecycle contract, not a promise of zero downtime or high availability.

### Bind configuration

The variables depend on the options selected when the app was created:

- **`--service` without `--settings` or `--vite`:** set `YETLET_HOST` and
  `YETLET_PORT`. The defaults are `127.0.0.1` and `3000`. The host accepts
  `127.0.0.1` or an explicitly selected `0.0.0.0`; the port must be a decimal
  value from `1` through `65535`.
- **`--service --settings` or `--service --vite`:** configure `server.host`
  and `server.port` in `base.toml` or the selected profile. Defaults are
  `127.0.0.1` and `3000`. Override declared settings with
  `APP_SERVER__HOST` and `APP_SERVER__PORT`. The same Service host and port
  validation applies. Settings-enabled startup also requires you to supply
  `APP_SERVICE__TOKEN` through your private runtime environment; no usable token
  is generated and credentials must not be placed in public TOML or frontend files.

Choose an externally reachable bind address only when the surrounding network,
firewall and deployment environment are configured appropriately. These routes
report this process's liveness/readiness; add application-specific dependency
checks separately if required.

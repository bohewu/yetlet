
## Application settings

The selected settings module loads typed server/service and known public CLI
settings, rejecting unknown fields. Set `APP_SERVICE__TOKEN` to your own value
before startup; no usable token is generated. The token never enters frontend
source or SSR. Keep credentials out of public TOMLs.

`YETLET_CONFIG_ROOT` can select an explicit absolute common settings directory.
The default remains this app's build-time `CARGO_MANIFEST_DIR`. Development
secret loading requires both `APP_ENVIRONMENT=Development` and an explicit
absolute `YETLET_DEV_SECRETS` path; no backend dotenv discovery occurs. CLI
public-settings selection never reads that secret file.

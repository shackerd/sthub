# sthub
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fshackerd%2Fsthub.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fshackerd%2Fsthub?ref=badge_shield)

*"(static)-hub"* — Your static and config hub, simplified

> **Note:** This project is under active development and not yet production-ready.
> For detailed documentation, see the [`docs/`](./docs/) folder or visit the [documentation site](https://shackerd.github.io/sthub/#/).

---

## What is it?

**sthub** is a lightweight, flexible static file server with built-in support for dynamic configuration via environment variables.
It is ideal for serving static web applications (such as SPAs) and exposing runtime configuration at a dedicated endpoint.

---

## Features

- **Serve static files** from a configurable directory and URL prefix.
- **Expose environment variables** as a structured JSON tree at a configurable endpoint (default `/env`).
- **Apache-style rewrite rules** for advanced static file routing (e.g., SPA fallback).
- **Custom response headers** for static file responses.
- **Configurable via YAML** (`conf.yaml`), including network, static, and configuration hubs.
- **CLI support** for specifying the configuration file path.

---

## Quick Start

1. **Configure your server** in `conf.yaml` (see [`docs/configuration.md`](./docs/configuration.md) for details).
2. **Run the server:**
   ```bash
   cargo run --release -- --configuration-path conf.yaml
   ```
   Or use the built binary.

3. **Access your static files** at the configured `remote_path` (e.g., `/public`).

4. **Fetch environment/configuration variables**:
   ```bash
   curl http://localhost:8080/env
   ```

---

## Configuration Overview

- **Network:** Set `host` and `port`.
- **Static hub:**
  - `remote_path`: URL prefix for static files (e.g., `/public`)
  - `path`: Directory to serve
  - `rewrite_rules`: Apache-style rules for routing (optional)
  - `headers`: Custom response headers (optional)
- **Configuration hub:**
  - `remote_path`: URL for environment/config endpoint (default `/env`)
  - `providers.env.prefix`: Prefix for environment variables (must end with `__`)

See [`docs/configuration.md`](./docs/configuration.md) for full details and examples.

---

## Environment Variable Notation

- Use double underscores (`__`) for nesting (e.g., `STHUB__DATABASE__HOST`).
- Arrays and objects are supported.
- See [`docs/environment_variable_notation.md`](./docs/environment_variable_notation.md) for more.

---

## License

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fshackerd%2Fsthub.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fshackerd%2Fsthub?ref=badge_large)

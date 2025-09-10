# Configuration Guide

This document explains how to configure **sthub** using the `conf.yaml` file.
It covers all major sections: `network`, `global`, and the different hub types (`static`, `configuration`, `upstream`).
You will also find details on rewrite rules, environment variable handling, and header customization.

---

## Example `conf.yaml`

```yaml
network:
  port: 8080
  host: "localhost"
  tls: # not implemented
    enabled: false
    cert_file: "path/to/cert.pem"
    key_file: "path/to/key.pem"

global:
  headers:
    "x-powered-by": sthub
    "foo": "bar"

hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    headers:
      "cache-control": "no-cache"
    rewrite_rules: |
      RewriteEngine On
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
      RewriteRule ^ /index.html

  configuration:
    remote_path: /conf
    cache: true # (not implemented)
    headers:
      "access-control-allow-origin": "none"
    providers:
      env:
        prefix: "STHUB"
      dotenv:
        path: ".env"

  upstream:
    target: 127.0.0.1:3000
    remote_path: /proxy_pass
```

---

## Section Details

### `network`
- **Purpose:** Configure the server’s host, port, and (future) TLS options.
- **Fields:**
  - `port`: Port to listen on (default: 8080)
  - `host`: Host address (default: "localhost")
  - `tls`: TLS options (not implemented)

### `global`
- **Purpose:** Define headers (and later, other options) applied to all responses.
- **Fields:**
  - `headers`: Map of header names and values (must be lowercase, ASCII, non-empty)

### `hubs.static`
- **Purpose:** Serve static files from a directory, with optional headers and rewrite rules.
- **Fields:**
  - `remote_path`: URL prefix for static files (e.g., `/`)
  - `path`: Directory to serve
  - `headers`: Custom headers for static responses
  - `rewrite_rules`: Apache-style rules for routing (optional)
- **Rewrite Rule Limitations:**
  Only simple prefix or equality matches are supported in rewrite rules. **Regex in conditions** (e.g., `RewriteCond %{REQUEST_URI} !\.(js|css)$`) is **not supported**.
  If you need to exclude asset directories, list them individually in your rules using path prefixes.

### `hubs.configuration`
- **Purpose:** Serve configuration/environment variables at a specific endpoint.
- **Fields:**
  - `remote_path`: URL for config endpoint (default: `/conf`, but configurable)
  - `cache`: Enable/disable caching (not implemented)
  - `headers`: Custom headers for this endpoint
  - `providers.env.prefix`: Prefix for environment variables
  - `providers.dotenv.path`: Path to `.env` file

### `hubs.upstream`
- **Purpose:** (Experimental) Intended for proxying requests to another backend.
- **Fields:**
  - `target`: Address of the upstream server
  - `remote_path`: URL prefix for proxying

---

## Additional Notes

- **Header Key Requirements:** All header keys must be lowercase, ASCII, and non-empty. Invalid keys/values are ignored.
- **Precedence:** Global headers apply to all responses; per-hub headers override global ones for their scope.
- **Not Implemented:** TLS, upstream, dotenv, and cache are placeholders for future features.
- **Endpoint Naming:** The configuration endpoint defaults to `/conf`, but you can set it to any other path using `remote_path` in your config.
- **Rewrite Rules:** See [Rewrite Rules](rewrite_rules.md) for more details and examples.
- **Dotenv Support:** See [Dotenv Support](environment_variable_notation.md#dotenv-support) for details on loading environment variables from a `.env` file.
- **For more details:** See [Custom Headers](headers.md) and [Environment Variable Notation](environment_variable_notation.md).

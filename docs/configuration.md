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
        prefix: "STHUB__"
      dotenv: # (not implemented)
        path: "./.env"
        hotreload: true

  upstream: # not implemented
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
- **Example:**
  ```yaml
  network:
    port: 8080
    host: "localhost"
    tls:
      enabled: false
      cert_file: "path/to/cert.pem"
      key_file: "path/to/key.pem"
  ```

### `global`
- **Purpose:** Define headers (and later, other options) applied to all responses.
- **Fields:**
  - `headers`: Map of header names and values (must be lowercase, ASCII, non-empty)
- **Example:**
  ```yaml
  global:
    headers:
      "x-powered-by": sthub
      "foo": "bar"
  ```

### `hubs.static`
- **Purpose:** Serve static files from a directory, with optional headers and rewrite rules.
- **Fields:**
  - `remote_path`: URL prefix for static files (e.g., `/`)
  - `path`: Directory to serve
  - `headers`: Custom headers for static responses
  - `rewrite_rules`: Apache-style rules for routing (optional)
- **Example:**
  ```yaml
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
  ```

### `hubs.configuration`
- **Purpose:** Serve configuration/environment variables at a specific endpoint.
- **Fields:**
  - `remote_path`: URL for config endpoint (e.g., `/conf`)
  - `cache`: Enable/disable caching (not implemented)
  - `headers`: Custom headers for this endpoint
  - `providers.env.prefix`: Prefix for environment variables (must end with `__`)
  - `providers.dotenv`: (not implemented) Path to `.env` file and hotreload option
- **Example:**
  ```yaml
  hubs:
    configuration:
      remote_path: /conf
      cache: true # (not implemented)
      headers:
        "access-control-allow-origin": "none"
      providers:
        env:
          prefix: "STHUB__"
        dotenv: # (not implemented)
          path: "./.env"
          hotreload: true
  ```

### `hubs.upstream`
- **Purpose:** (Not implemented) Intended for proxying requests to another backend.
- **Fields:**
  - `target`: Address of the upstream server
  - `remote_path`: URL prefix for proxying
- **Example:**
  ```yaml
  hubs:
    upstream:
      target: 127.0.0.1:3000
      remote_path: /proxy_pass
  ```

---

## Additional Notes

- **Header Key Requirements:** All header keys must be lowercase, ASCII, and non-empty. Invalid keys/values are ignored.
- **Precedence:** Global headers apply to all responses; per-hub headers override global ones for their scope.
- **Not Implemented:** TLS, upstream, dotenv, and cache are placeholders for future features.
- **For more details:** See [Custom Headers](headers.md) and [Environment Variable Notation](environment_variable_notation.md).

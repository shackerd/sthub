# Configuration Guide

This document explains how to configure static file serving and rewrite rules in **sthub**. You can choose between two modes: a simple "try files" mode or a flexible rewrite mode using Apache-style rules.

---

## Modes of Operation

### 1. Try Files Mode (Default)

- **Behavior:**
  The server attempts to serve the requested file.
  If the file does not exist, it serves `index.html` instead (ideal for Single Page Applications).
- **How to enable:**
  Omit the `rewrite_rules` section in your configuration.

### 2. Rewrite Mode

- **Behavior:**
  The server uses Apache-style rewrite rules to determine how requests are routed.
  This allows for advanced routing, SPA fallback, redirects, and more.
- **How to enable:**
  Add a `rewrite_rules` section under your static hub configuration in `conf.yaml`.
- **Note:**
  Comments (lines starting with `#`) are not supported in the rewrite rules. Only use valid rewrite directives.

---

## Example `conf.yaml`

```yaml
network:
  port: 8080
hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    # To enable rewrite mode, provide rewrite_rules:
    rewrite_rules: |
      RewriteEngine On
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
      RewriteRule ^ /index.html
```

---

## Matching `remote_path` with Rewrite Rules

When configuring a static hub in your `conf.yaml`, the `remote_path` determines the URL prefix that the hub will handle. **All rewrite rules you define must match requests that begin with this `remote_path`.** If a rewrite rule does not match the `remote_path`, it will not be applied to incoming requests.

### How it works

- Only requests matching this prefix are processed by the hub and its rewrite rules.
- Rewrite rules are then applied to the path **after** the `remote_path` prefix.

#### Example

Given this configuration:
```yaml
hubs:
  static:
    remote_path: /public
    path: "/var/www/html/"
    rewrite_rules: |
      RewriteEngine On
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
      RewriteRule ^ /public/index.html
```

- Only requests starting with `/public` (e.g., `/public/app.js`, `/public/anything`) will be handled by this hub.
- The rewrite rule `RewriteRule ^ /public/index.html` will only apply to requests under `/public`.
- Requests to `/other` or `/env` will **not** be affected by these rules.

#### Important

- **Always ensure your rewrite rules target paths that include the `remote_path` prefix.**
- If you want to rewrite all unknown routes under `/public` to `/public/index.html`, your rule should be `RewriteRule ^ /public/index.html` (not just `/index.html`).

---


## Configuration Options

| Option         | Description                                                                                  |
|----------------|---------------------------------------------------------------------------------------------|
| `path`         | Directory to serve static files from.                                                        |
| `rewrite_rules`| (Optional) Apache-style rules. If present, enables rewrite mode.                             |

- If `rewrite_rules` is omitted or empty, the server uses Try Files mode.
- If `rewrite_rules` is present, the server uses Rewrite mode.

---

## Notes & Recommendations

- **For SPAs:**
  Use rewrite rules to ensure all unknown routes are served `index.html`.
  The correct logic for SPA fallback is to rewrite when the requested file or directory does **not** exist, as shown in the example above.
- **Customization:**
  You can fully customize the rewrite rules for advanced routing needs.
- **Static Assets:**
  Static files (like JS, CSS, images) are always served directly if they exist.

---

## Troubleshooting

- **"Rewrite generated an invalid uri":**
  Ensure your rewrite targets (e.g., `/index.html`) start with a `/` and are valid paths.
- **Debugging:**
  Enable logging to see how requests are being rewritten and which files are being served.
- **Comments not supported:**
  Do not include comment lines (starting with `#`) in your rewrite rules section.
- **Regex limitations:**
  Only simple prefix or equality matches are supported in rewrite rules. Advanced regex features like lookahead/lookbehind are **not supported**.
---

For further details or advanced configuration, see the main documentation or contact the maintainers.

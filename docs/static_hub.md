# Static Hub Configuration

The **static hub** in `sthub` is responsible for serving static files from a specified directory, with support for custom response headers and flexible routing via rewrite rules. This document explains how to configure and use the static hub in your `conf.yaml`.

---

## Overview

The static hub allows you to:
- Serve files (HTML, JS, CSS, images, etc.) from a directory at a configurable URL prefix.
- Add custom HTTP headers to all static file responses.
- Use Apache-style rewrite rules for advanced routing (e.g., SPA fallback).

---

## Example Configuration

```yaml
hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    headers:
      "cache-control": "no-cache"
      "x-custom-header": "my-value"
    rewrite_rules: |
      RewriteEngine On
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
      RewriteRule ^ /index.html
```

---

## Configuration Options

| Option         | Description                                                                                  |
|----------------|---------------------------------------------------------------------------------------------|
| `remote_path`  | The URL prefix where static files are served (e.g., `/` or `/public`).                      |
| `path`         | The directory on disk containing your static files.                                          |
| `headers`      | (Optional) Map of custom HTTP headers for static responses. Keys must be lowercase, ASCII.   |
| `rewrite_rules`| (Optional) Apache-style rewrite rules for advanced routing.                                  |

---

## Custom Headers

You can add custom headers to all static file responses using the `headers` field.
**Header keys must be lowercase, ASCII, and non-empty.**
Example:

```yaml
headers:
  "cache-control": "no-cache"
  "x-powered-by": "sthub"
```

For more on header requirements and merging with global headers, see [Custom Headers](headers.md).

---

## Rewrite Rules

The `rewrite_rules` option allows you to define Apache-style rules for advanced routing scenarios, such as Single Page Application (SPA) fallback.

**Example: SPA fallback**
```yaml
rewrite_rules: |
  RewriteEngine On
  RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
  RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
  RewriteRule ^ /index.html
```
- This configuration serves `index.html` for any request that does not match an existing file or directory.

**Notes:**
- Rewrite rules are only applied to requests matching the `remote_path`.
- Only simple prefix or equality matches are supported; advanced regex features are not.
- Comments (lines starting with `#`) are not supported in the rewrite rules section.

---

## Usage Tips

- Place your static assets (HTML, JS, CSS, images) in the directory specified by `path`.
- Set `remote_path` to `/` to serve files at the root, or to `/public` to serve under a subpath.
- Use rewrite rules for SPA routing or custom URL handling.
- Combine with [global headers](configuration.md#global) for consistent HTTP headers across your app.

---

## Troubleshooting

- **Files not found:** Ensure the `path` points to the correct directory and files exist.
- **Headers not applied:** Check that header keys are lowercase and ASCII.
- **Rewrite not working:** Make sure your rules are valid and the request matches `remote_path`.

---

For more details on the full configuration file, see the [Configuration Guide](configuration.md).

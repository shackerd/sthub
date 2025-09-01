# Static Hub & Rewrite Rules

This document explains how to configure and use rewrite rules with the **static hub** in `sthub`. Rewrite rules allow you to control how requests for static files are routed, enabling advanced use cases such as Single Page Application (SPA) fallback, custom routing, and more.

---

## What Are Rewrite Rules?

Rewrite rules let you define how incoming URLs are mapped to files on disk or to other URLs. This is especially useful for SPAs, where you want all unknown routes to serve `index.html`, or for custom URL routing logic.

Rewrite rules in `sthub` use an Apache-like syntax, familiar to many web developers.

---

## Enabling Rewrite Rules

To enable rewrite rules, add a `rewrite_rules` section under your static hub configuration in `conf.yaml`:

```yaml
hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    rewrite_rules: |
      RewriteEngine On
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
      RewriteRule ^ /index.html
```

If `rewrite_rules` is omitted, the server defaults to a "try files" mode: it serves the requested file if it exists, otherwise it returns a 404 or the default document.

---

## Syntax Reference

- **RewriteEngine On**
  Enables the rewrite engine. This line is required.

- **RewriteCond**
  Adds a condition for the following `RewriteRule`.
  Example:
  `RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f`
  Only rewrite if the requested file does not exist.

- **RewriteRule**
  The main rule for rewriting URLs.
  Syntax:
  `RewriteRule <pattern> <target>`
  - `<pattern>`: Regex pattern to match the request path (relative to `remote_path`).
  - `<target>`: The path to serve if the rule matches.

**Note:**
- Only simple prefix or equality matches are supported. Advanced regex features (lookahead/lookbehind) are **not supported**.
- Comments (lines starting with `#`) are not supported in the `rewrite_rules` section.

---

## SPA Fallback Example

For Single Page Applications (React, Angular, Vue, etc.), you typically want all unknown routes to serve `index.html`.
Here is the recommended configuration:

```yaml
hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    rewrite_rules: |
      RewriteEngine On
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-f
      RewriteCond %{DOCUMENT_ROOT}%{REQUEST_URI} !-d
      RewriteRule ^ /index.html
```

**Explanation:**
- If the requested path does not match a file (`!-f`) or directory (`!-d`), rewrite the request to `/index.html`.

---

## Custom Routing Example

You can use rewrite rules to serve custom files for specific routes:

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

---

## Matching `remote_path`

- **IMPORTANT:** If you set a `remote_path`, your rewrite rules MUST match the path after the `remote_path` prefix.
- Rewrite rules are only applied to requests that begin with the `remote_path` specified in your static hub.
- The pattern in `RewriteRule` should match the path **after** the `remote_path` prefix.
- For example, if your `remote_path` is `/public`, then a request to `/public/app.js` will be matched as `/app.js` in the rule.
- If your rule expects `/public/index.html` but your `remote_path` is `/public`, you must write the rule as `RewriteRule ^ /public/index.html` (not `/index.html`).

**Warning:** If your rewrite rules do not match the path after the `remote_path` prefix, they will not be applied as expected.

---

## Troubleshooting & Tips

- **"Rewrite generated an invalid uri":**
  Ensure your rewrite targets (e.g., `/index.html`) start with a `/` and are valid paths.
- **Debugging:**
  Enable logging to see how requests are being rewritten and which files are being served.
- **No comments:**
  Do not include comment lines (starting with `#`) in your `rewrite_rules` section.
- **Order matters:**
  Rules are evaluated in order. Place more specific rules before generic ones.

---

## Further Reading

- See the [Configuration Guide](configuration.md) for a full overview of all configuration options.
- For header customization, see [Custom Headers](headers.md).

---

# Custom Response Headers

**sthub** allows you to customize HTTP response headers globally (for all responses) and per hub (for specific endpoints such as static files or configuration). This enables you to add security headers, custom server identifiers, or any other headers required by your application or deployment environment.

---

## Global Headers

You can define headers that will be added to every HTTP response, regardless of the hub or endpoint, by using the `global.headers` section in your `conf.yaml`.

**Example:**
```yaml
global:
  headers:
    "x-powered-by": sthub
    "foo": "bar"
```

---

## Header Key Requirements

- **Header names must be lowercase, ASCII-only, and non-empty.**
- Hyphens (`-`) are allowed, but spaces, underscores, and uppercase letters are not.
- If a header key or value is invalid or empty, it will be ignored and not added to the response.
- If you provide an invalid header key (e.g., empty string, uppercase, or non-ASCII), it will be skipped and may be logged as a warning.

**Examples of valid header keys:**
- `x-powered-by`
- `cache-control`
- `x-frame-options`

**Examples of invalid header keys (will be ignored):**
- `X-Powered-By` (uppercase)
- `X_Powered_By` (underscore)
- `` (empty string)
- `x-éxample` (non-ASCII)

**Always use lowercase and hyphens for best compatibility.**

These headers will be present on all responses, unless overridden by more specific hub-level headers.

---

## Notes

- Header names are case-insensitive in HTTP, but **you must use lowercase and hyphens in your configuration** for compatibility with the server.
- If a header is already set by the server or another middleware, your custom value will override it.
- Invalid or empty header keys/values will be ignored.
- Global headers apply to all responses unless overridden by per-hub headers.

---

## Use Cases

- **Security:** Add headers like `x-frame-options`, `x-content-type-options`, or `strict-transport-security`.
- **Caching:** Control browser and proxy caching with `cache-control` or `expires`.
- **Branding:** Set a custom `server` or `x-powered-by` header.
- **Custom Needs:** Add any other headers required by your application or infrastructure.

---

For more details on configuration options, see the [Configuration Guide](configuration.md).

## Per-Hub Headers

You can also define headers for a specific hub (such as `static` or `configuration`). These headers are only applied to responses handled by that hub.

**Example for static files:**
```yaml
hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    headers:
      "cache-control": "no-cache"
```

**Example for configuration endpoint:**
```yaml
hubs:
  configuration:
    remote_path: /conf
    headers:
      "access-control-allow-origin": "none"
```

---

## Precedence and Merging

- **Global headers** are always applied first.
- **Per-hub headers** (if defined) are merged on top of global headers for responses handled by that hub. If a header key exists in both, the per-hub value takes precedence.
- If neither is defined, no custom headers are added.

---

## How It Works

The `headers` option in your `conf.yaml` static hub configuration lets you specify a map of header names and values. These headers will be added to every response for static files served under the configured `remote_path`.

---

## Example Configuration

```yaml
hubs:
  static:
    remote_path: /
    path: "/var/www/html/"
    headers:
      "Server": sthub
      "X-Frame-Options": DENY
      "X-Content-Type-Options": nosniff
      "Cache-Control": "public, max-age=3600"
```

In this example:
- All static file responses under `/` will include the specified headers.
- You can use any valid HTTP header name and value.

---

## Header Key Requirements

- **Header names must be lowercase, ASCII-only, and non-empty.**
- Hyphens (`-`) are allowed, but spaces, underscores, and uppercase letters are not.
- If a header key or value is invalid or empty, it will be ignored and not added to the response.
- If you provide an invalid header key (e.g., empty string, uppercase, or non-ASCII), it will be skipped and may be logged as a warning.

**Examples of valid header keys:**
- `x-powered-by`
- `cache-control`
- `x-frame-options`

**Examples of invalid header keys (will be ignored):**
- `X-Powered-By` (uppercase)
- `X_Powered_By` (underscore)
- `` (empty string)
- `x-éxample` (non-ASCII)

**Always use lowercase and hyphens for best compatibility.**

---

## Notes

- Header names are case-insensitive in HTTP, but **you must use lowercase and hyphens in your configuration** for compatibility with the server.
- If a header is already set by the server or another middleware, your custom value will override it.
- This feature is only available for the static hub (not for dynamic endpoints like `/env`).
- Invalid or empty header keys/values will be ignored.

---

## Use Cases

- **Security:** Add headers like `X-Frame-Options`, `X-Content-Type-Options`, or `Strict-Transport-Security`.
- **Caching:** Control browser and proxy caching with `Cache-Control` or `Expires`.
- **Branding:** Set a custom `Server` header.
- **Custom Needs:** Add any other headers required by your application or infrastructure.

---

For more details on configuration options, see the [Configuration Guide](configuration.md).

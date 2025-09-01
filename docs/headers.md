# Custom Response Headers

You can customize the HTTP response headers for static files served by **sthub**. This feature allows you to add security headers, custom server identifiers, or any other headers required by your application or deployment environment.

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

# Environment Variable Notation Guide

This system allows you to define complex, nested configuration structures using environment variables. The variables are parsed into a JSON tree, making it easy to represent objects and arrays in your configuration.

## Notation Rules

- **Prefix**: All environment variables must start with a specific prefix ending in a double underscore (`__`).  
  Example: `STHUB__`

- **Separator**: Use double underscores (`__`) to indicate nesting (object keys or array indices).

### Objects

To represent nested objects, use the separator between each level:

- `STHUB__DATABASE__HOST=localhost`  
  → `{ "DATABASE": { "HOST": "localhost" } }`

- `STHUB__DATABASE__PORT=5432`  
  → `{ "DATABASE": { "PORT": "5432" } }`

### Arrays

To represent arrays, use numeric keys as the last segment:

- `STHUB__SERVERS__0=alpha`  
- `STHUB__SERVERS__1=beta`  
- `STHUB__SERVERS__2=gamma`  
  → `{ "SERVERS": ["alpha", "beta", "gamma"] }`

### Mixed Objects and Arrays

If a structure mixes numeric and non-numeric keys, it will be treated as an object:

- `STHUB__MIXED__0=zero`  
- `STHUB__MIXED__NAME=name_value`  
  → `{ "MIXED": { "0": "zero", "NAME": "name_value" } }`

### Non-Consecutive Indices

If array indices are not consecutive (e.g., 0 and 2), the structure is treated as an object:

- `STHUB__SPARSE__0=zero`  
- `STHUB__SPARSE__2=two`  
  → `{ "SPARSE": { "0": "zero", "2": "two" } }`

## Example

Given these environment variables:

- `STHUB__API__URL=https://example.com`
- `STHUB__API__KEY=secret`
- `STHUB__FEATURES__0=featureA`
- `STHUB__FEATURES__1=featureB`

The resulting JSON would be:

```json
{
  "API": {
    "URL": "https://example.com",
    "KEY": "secret"
  },
  "FEATURES": ["featureA", "featureB"]
}
```

---
This notation enables you to easily manage complex configuration hierarchies using only environment variables, making your application's configuration flexible and portable.
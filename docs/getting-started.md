# Getting started

The server exposes prefixed environment variables as a JSON object at the `/conf` endpoint by default. This path can be configured in your `conf.yaml`.

The prefix for environment variables is configurable (default: `STHUB`). You can set it in your configuration file under `providers.env.prefix`.

## How to use it?

Run the server with the following command:

```bash
sthub -c ./conf.yaml
```

## Integration

You can request environment variables from the server by making a GET request to the `/conf` endpoint.

The server will respond with a JSON object containing the environment variables.

Example of a GET request to `/conf` endpoint in JavaScript:

```javascript
fetch('/conf')
  .then(response => response.json())
  .then(env => {
    console.log('Environment variables loaded:', env);
    // Use the environment variables in your application
  })
  .catch(error => {
    console.error('Error loading environment variables:', error);
  });
```

## Example

```bash
export MYAPI__ENV_VAR="Hello, World"
export MYAPI__SUBLEVEL__ENV_VAR="Another value"
sthub -c ./conf.yaml
```

Then, you can view environment variables by making a GET request to `/conf` endpoint:

```bash
curl -s http://127.0.0.1:8080/conf | jq
```

## CORS

By design, if you serve your static files from the same origin as the server, you will not have any CORS issues when requesting the `/conf` endpoint.

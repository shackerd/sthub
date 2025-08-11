# Getting started

The server is a simple HTTP server that serves environment variables from the system.

The server will expose prefixed environment variables as a JSON object at the `/env` endpoint, this is a security measure.

Actually the prefix is `MYAPI__` but will be settable in a configuration file or in CLI in future versions.


## How to use it?

Run the server with the following command:

```bash
sthub #(cli commands not yet implemented)
```

## Integration

You can request environment variables from the server by making a GET request to the `/env` endpoint.

The server will respond with a JSON object containing the environment variables.

Example of a GET request to `/env` endpoint in javascript:

```javascript

fetch('/env')
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
sthub #(cli commands not yet implemented)
```

Then, you can set environment variables by making a POST request to `/env` endpoint:

```bash
curl -s http://127.0.0.1:8080/env | jq
```

## CORS

By design if you serve your static files from the same origin as the server, you will not have any CORS issues when requesting the `/env` endpoint.

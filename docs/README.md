# sthub
*"(static)-hub"* — Your static and config hub, simplified

## Disclaimer

This project is subject to change and may not be stable. It is not recommended to use it in production environments yet. Use at your own risk.

## What is it?
A simple static file server that serves files from a directory and allows you to configure it with a simple HTTP request to the `/conf` path (default, configurable). Useful for web applications (e.g. Angular applications, static websites, etc.).

It is designed to be used as a static file server for web applications, but can be used for any static file serving needs. It also supports dynamic configuration via environment variables and a dedicated endpoint.

## Goal

Bring simplicity to static file serving, allowing you to focus on your application rather than environments. This project aims to provide flexibility to the delivery process by allowing you to remove static configuration files from the application codebase.

## Limitations

This project is in early development stages and may not have all the features you need. It is not yet ready for production use, but it is a good starting point for serving static files and environment variables.

For advanced features (compression, caching, HTTPS), you may use a reverse proxy (e.g. nginx) in front of it to serve static files from a directory for better performance and feature needs.

# sthub
*"(static)-hub"* — Your static and config hub, simplified


## Disclaimer

This project is subject to change and may not be stable. It is not recommended to use it in production environments yet. Use at your own risk.

## What is it?
A simple static file server that serves files from a directory and allows you to configure it with a simple HTTP request to `/env` path. Useful for web applications (e.g. angular applications, static websites, etc.).

It is designed to be used as a static file server for web applications, but can be used for any static file serving needs.
It is a simple, lightweight, and easy-to-use solution for serving static files and configuration.

## Goal

Bring simplicity to static file serving, allowing you to focus on your application rather than environments. This project aims to provide flexibility to the delivery process by allowing to remove static configuration files from the application codebase.

## Limitations

This project is in early development stages and may not have all the features you need. It is not yet ready for production use, but it is a good starting point for serving static files and environment variables.
For the moment, this project is not intended to be a full-fledged static file server, but rather a simple solution for serving static files and environment variables, you may use a reverse proxy (e.g. nginx) in front of it to serve static files from a directory for better performance and feature needs.

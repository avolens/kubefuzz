---
sidebar_position: 3
---

# Logging

KubeFuzz can be configured to emit logging messages in various verbosity levels using the `RUST_LOG` environment variable. By default the
level is `info`. The available levels are

- Error
- Warn
- Info
- Debug
- Trace

## Example

`RUST_LOG=debug kubefuzz generate...`

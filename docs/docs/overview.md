---
sidebar_position: 2
---

# Overview

Kubefuzz is a generative and mutative fuzzer for Kubernetes admission controller chains. It can be used to uncover unexpected behavior in complex admission controller setups. It works by generating and mutating Kubernetes resources according to the schema supplied by the cluster openapi scheme, and a user written constrain configuration that further limits what fields are generated and how.

## Modes of operation 

```terminal
user@lnx ~> kubefuzz --help
Usage: kubefuzz [OPTIONS] <COMMAND>

Commands:
  generate     generate manifests with constraints
  mutate       mutate existing manifests with constraints
  fuzz         fuzz admission controller chain with constraints
  get-schemas  get json schemas from k8s api
  help         Print this message or the help of the given subcommand(s)

Options:
  -s, --seed <SEED>  seed to use
  -h, --help         Print help
  -V, --version      Print version
```

Kubefuzz can be run in 4 modes. To be able to use fuzzing, understanding the `fuzz` and the `get-schemas` mode and `constraints` is required.


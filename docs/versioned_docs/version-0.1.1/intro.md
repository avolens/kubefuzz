---
sidebar_position: 1
title: Introduction
---

# Introduction

## Building and Installing

First clone the repo and `cd` into it:

```
git clone https://github.com/avolens/kubefuzz
cd kubefuzz
```

Now select a release version from the available git tags. You can list them by doing

```
git tag
```

and select a tag with

```
git checkout <tagname>
```

To run the latest development build, which might have breaking changes, just stay on the most recent commit. When running Kubefuzz, you will notice the version is replaced by "dev".

Now you can now either just build the project by running

`cargo build -r ` or directly install it to your configured cargo binary path by issuing
`cargo install --path`.

## Additional dependencies

To pull schemas form the cluster api, you will need to `pip install openapi2jsonschema`

## Overview of Kubefuzz

Kubefuzz is a generative and mutative fuzzer for Kubernetes admission controller chains. It can be used to uncover unexpected behavior in complex admission controller setups. It works by generating and mutating Kubernetes resources according to the schema supplied by the cluster openapi scheme, and a user written constrain configuration that further limits what fields are generated and how.

### Modes of operation

```terminal
usr@lnx ~> kubefuzz --help
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

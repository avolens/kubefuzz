---
sidebar_position: 4
---

# Fuzzing mode

Fuzzing mode combines mutation and generation in a loop to test the K8s admission controller chain by reading the feedbacak
from the API. In addition to the usual constraint config(s) you'll have to specify the namespace you want the resources to be
dry run submitted in. Further, KubeFuzz will try to load the K8s config from default locations but you can also specify a custom
K8s config via `--path`

KubeFuzz will submit resources with the `dryrun` flag set. This will ensure that **the resources aren't actually deployed** but the
admission controller chain is hit regardless.

KubeFuzz will save resources that were accepted and resources that caused an error in an admission controller in the fuzzing directory.

You can quit the TUI that will show when fuzzing wiht ctrl-c.


```terminal
usr@lnx ~/kubefuzz> kubefuzz fuzz --help
fuzz admission controller chain with constraints

Usage: kubefuzz fuzz [OPTIONS] --constraints <CONSTRAINTS> --schemadir <SCHEMADIR> --fuzzdir <FUZZDIR>

Options:
  -k, --kubeconfig <KUBECONFIG>
          optional custom path to kubeconfig
  -c, --constraints <CONSTRAINTS>
          comma seperated list of constraint files to apply
  -s, --schemadir <SCHEMADIR>
          directory containing k8s json resource schemas
  -n, --namespace <NAMESPACE>
          namespace to use while fuzzing [default: default]
  -f, --fuzzdir <FUZZDIR>
          directory to save and update fuzzing results
  -t, --timeout <TIMEOUT>
          time in seconds until an api request is considered timed out [default: 5]
  -m, --max-corpus-count <MAX_CORPUS_COUNT>
          max number of samples into in memory coprus [default: 100]
  -a, --max-accepted <MAX_ACCEPTED>
          max number of accepted samples saved into fuzzing directory [default: 50]
  -e, --max-error <MAX_ERROR>
          max number of error causing samples saved into fuzzing directory [default: 50]
  -i, --iterations <ITERATIONS>
          number of fuzzing iterations to perform, 0 = infinite [default: 0]
  -g, --generations <GENERATIONS>
          number of new generations per iteration per constraint [default: 20]
      --notui
          disable the TUI
  -h, --help
          Print help

usr@lnx ~> kubefuzz fuzz -c src/constraint.yaml  -s src/schemagen/schemas/  --fuzzdir /tmp/out/
```

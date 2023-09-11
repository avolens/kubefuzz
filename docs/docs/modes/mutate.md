---
sidebar_position: 3
---

# Mutation mode

In mutation mode KubeFuzz will mutate existing resources according to the user supplied constraint:

```terminal
usr@lnx ~> kubefuzz  mutate --help
mutate existing manifests with constraints

Usage: kubefuzz mutate [OPTIONS] --resources <RESOURCES> --schemadir <SCHEMADIR> --out <OUT> --constraints <CONSTRAINTS>

Options:
  -r, --resources <RESOURCES>      comma seperated list of resources to be mutated
  -s, --schemadir <SCHEMADIR>      directory containing k8s json resource schemas
  -o, --out <OUT>                  output directory of mutated resources
  -n, --num <NUM>                  number of mutated resources to generate per resource [default: 10]
  -c, --constraints <CONSTRAINTS>  comma seperated list of constraint files to apply
  -m, --max-samples <MAX_SAMPLES>  max number of samples saved into fuzzing directory [default: 50]
  -h, --help                       Print help

usr@lnx ~> kubefuzz mutate -s src/schemagen/schemas/ -c constraint.yaml -o /tmp/ -r resources/0.yaml,resources/1.yaml
 INFO  kubefuzz > running wiht seed 5048611348397963138
 INFO  kubefuzz::generator > Reading constraint file: "..."
 INFO  kubefuzz::generator > Reading spec file: "..."
 WARN  kubefuzz::generator > overriding enum for field '$.spec.containers.securityContext.seccompProfile.type', original content : [String("Localhost"), String("RuntimeDefault"), String("Unconfined")]
 INFO  kubefuzz::runtime::mode_mutate > Done mutating. Written 20 files
```

---
sidebar_position: 5 
---

# Generation mode 

In generation mode KubeFuzz will generate random resources according to the user supplied constraint:

```terminal
user@lnx ~> kubefuzz generate --help
generate manifests with constraints

Usage: kubefuzz generate [OPTIONS] --constraints <CONSTRAINTS> --schemadir <SCHEMADIR> --out <OUT>

Options:
  -c, --constraints <CONSTRAINTS>  comma seperated list of constraint files to apply
  -s, --schemadir <SCHEMADIR>      directory containing k8s json resource schemas
  -o, --out <OUT>                  output direcotry of generated schemas
  -n, --num <NUM>                  number of manifests to generate per resource [default: 10]
  -h, --help                       Print help

user@lnx ~> mkdir out
user@lnx ~> kubefuzz generate -c /path/to/constraint.yaml,anotherconstraint.yaml -o out -s /path/to/schemas/
```



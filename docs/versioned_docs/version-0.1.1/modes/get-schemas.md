---
sidebar_position: 1
---

# Get-schemas mode

In order for KubeFuzz to be able to work with the resources existent your cluster, it needs to know their basic structure. This includes for example required fields and value formats. With this knowledge, Kubefuzz can smartly generate random resources that still comply with the resource definition in a syntactic and semantic correct way (see known issues for edge cases).

Kubernetes automatically exposes resource schemas for every resource present in the cluster under the `/openapi/v2` endpoint. KubeFuzz uses *openapi2json* internally and *needs access to the openapi cluster endpoint*

Currently, this is achieved by running

```
kubectl proxy
```

This will create a http interface which requires no authentication. Kubefuzz can now be pointed at this endpoint to pull all schemas from the cluster:

```
usr@lnx ~> kubectl proxy &
Starting to serve on 127.0.0.1:8001
usr@lnx ~> kubefuzz get-schemas --endpoint http://127.0.0.1:8001/openapi/v2
Downloading schema
Parsing schema
Generating shared definitions
Generating individual schemas
Processing mutatingwebhook
Generating mutatingwebhook.json
Processing mutatingwebhookconfiguration
Generating mutatingwebhookconfiguration.json
Processing mutatingwebhookconfigurationlist
Generating mutatingwebhookconfigurationlist.json
Processing rulewithoperations
Generating rulewithoperations.json
Processing servicereference
...
```

This will download all schemas to the `schemas` folder in your current working directory.

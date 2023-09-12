---
sidebar_position: 5
---

# Example

## Create a kind Cluster

1. Create the [kind](https://kind.sigs.k8s.io/) cluster: `kind create cluster --name kubefuzz`
2. Wait until everything is up and running: `kubectl get pods -A`

## Deploy the sample Admission Controller

1. create the kubefuzz namespace: `kubectl create namespace kubefuzz`
2. deploy the custom resource: `kubectl apply -f fooddeliveryorder-crd.yaml`
3. deploy the admission controller: `kubectl apply -f deployment.yaml`
4. deploy the admission controller config: `kubectl apply -f validatingac.yaml`


## Use Kubefuzz

1. get the current schemas from the k8s api (refer to docs or kubefuzz --help)
2. toy around with kubefuzz: `kubefuzz fuzz -f fuzzdir -s schemas -c constraint.yaml -e 250`

## Predfined Constraints

You can find predefined Constraints in the [predefined_constraints](https://github.com/avolens/kubefuzz/tree/master/sample_constraints) folder in the Github repo.

# Sample Admission Controller

This directory contains a sample admission controller operating on a sample CRD which can be fuzzed to get fammiliar with KubeFuzz. A starting sample constraint is also
provided. The sample admission controller can be deployed from the cloud and does not have to build locally.

The sample admission controller contains some logic and runtime bugs documented in its short source code. This example uses a custom resource that represents
a simple food delivery order.

You can also build the AC yourself by refering to the dockerfile.

## Starting

1. create the kubefuzz namespace: `kubectl namespace create kubefuzz`
2. deploy the custom resource: `kubectl apply -f fooddeliveryorder-crd.yaml`
3. deploy the admission controller: `kubectl apply -f deployment.yaml`
4. deploy the admission controller config: `kubectl apply -f validatingac.yaml`
5. get the current schemas from the k8s api (refer to docs or kubefuzz --help)
6. toy around with kubefuzz: `kubefuzz fuzz -f fuzzdir -s schemas -c constraint.yaml -e 250`

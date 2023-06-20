# Kubefuzz

![](arch/architecture.drawio.png)

Kubefuzz is a generative and mutative fuzzer for kubernetes admission controller chains.
It can be used to uncover unexpected behavior in complex admission controller setups. It works
by generating and mutating kubernetes resources according to the schema supplied by the cluster
openapi scheme, and a user written constrain config that further limits what fields are generated
and how.

# Building

```
git clone https://github.com/avolens/kubefuzz
cd kubefuzz
cargo build -r 
```

# Documenation

Documenation is available at [](https://kubefuzz.kubernetes-security.com/)

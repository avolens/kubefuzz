# Kubefuzz

![](arch/architecture.drawio.png)

Chain of validating and mutating admission controllers can become complex. **What manifests might unexpectedly get accepted/rejected?**

## Challenges to solve

- Generate meaningful manifests: pods,namespaces,volumes,deployments... Also do that in a general way. We want to avoid manual work
- AC instrumentation. Get *optional* feedback from AC's thats more than accept/rejec
- Configuration. Users should have control over what fields of a manifest type will get mutated and which are constant

## The mutator

- Input: Kubernetes resource scheme, User config with fields to fuzz
- Output: mutated manifests

## Future Ideas

Fuzz CIL language directly

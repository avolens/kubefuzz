---
sidebar_position: 8 
---

# Tipps

To make fuzzing more effective there are some things you might want to be careful about.

## Admission Controller Messages

KubeFuzz will process AC messages as part of the coverage information. This means that every
unique AC message will cause a new coverage event when fuzzing. When an AC emits random or
input dependant messages like `pod <podname> may not have be privileged` (the podname may be random),
it will cause a lot of "new" coverage which may lead to the eviction of actual useful coverage events
in the case of low free corpus capacity (you can change the in memory corpus capacity with the `--max-corpus-count`
flag). To counter this problem, it is advised to use generic, static messages for each cause of denial such as 
`one pod was privileged`

## Reproducability and Statefullness

When you discover an input that gets accepted or even crashes the admission controller,
the natural next step would be to verify this effect. As AC chains may get very complex,
there might be a state shared between controllers or internal states in ACs. This
could lead to reproducibility issues as the state has likely changed multiple times during
fuzzing.

## Side Effects

Be careful about admission controller side effects, such as file writes (like logging)
or network communication. KubeFuzz will ensure that the `dryrun` flag is added on every
request but cannot make the admission controllers behave completely side effect free.

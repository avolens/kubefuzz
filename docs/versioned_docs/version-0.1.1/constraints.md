---
sidebar_position: 4 
---

# Constraints

Kubefuzz does not blindly generate resources based on their specs (but you could configure it that way) but rather follows a constraint that the user
can supply, allowing fine grained control over which fields are generated and how

Generally, constraining works in a allow listing fashion. Every path that is not explicitly stated, is not generated (but every subpath of a specified
path *is* generated, if not otherwise explicitly disallowed)

Constraints are created using a specific format which will be described here. One constraint configuration per resource type is required.

## General format

Constraint configurations can be written in yaml or json. This guide will use yaml. Up front, here is an example constraint configuration for `core.v1.Pod`

```yaml 
group: ""
version: "v1"
kind: "Pod"

fields:
- "$.spec.containers.securityContext" # simple paths without any extras, just the path as a string

- path: "$.spec.containers"
  minmax: [1,3] # 1 to 3 containers will be generated, only works on arrays

- path: "$.spec.containers.name"
  required: true # will always be generated

- path: "$.spec.containers.securityContext.seccompProfile.localhostProfile"
  remove: true

- path: .*\.containers\..*gmsaCredentialSpecName # applies to every path matching this regex
  regex: true # dont forget to declare this path entry as a regex
  values: # every occurence of this field will have a value of either static or abc
    - "static"
    - "abc"
  values_mode: override # if there is an enum present (preset of possible values), override

- path: "$.spec.containers.image"
  required: true
  regex_values:  # values can also be randomly generated by a regex
    - "regex-img-name-[a-Z]{3}\d{3}"
    - "a-second-regex"

# every generated pod is guaranteed to
# 1. have every container have a securitycontext
# 2. have .privileged set to true
- path: "$.spec.containers.securityContext.privileged"
  values:
  - true
  required: true
```

### Required fields

You need to at least set the group, version, kind and one entry in the fields array in order for KubeFuzz to work. The `gvk`
triple specifies which resource to apply the constraint to. Note in the above example, the resource is `core.v1.pod`
but the `core` group is actually implicitly set.

### Field array

The field array contains a list of configurations that apply to one or multiple (if its a regex path) fields per item. Right up front, here are all possible fields for one such item in the array:

```rust
pub struct FieldConfig {
    pub path: String,
    pub values: Option<Vec<serde_yaml::Value>>,
    pub regex_values: Option<Vec<String>>,
    pub values_mode: Option<ValuesMode>,
    pub required: bool,
    pub minmax: Option<(usize, usize)>,
    pub remove: bool,
    pub regex: bool,
}
```

#### Path, regex and required

If you just want to enable a field to be generated, you can also just pass the field path instead of the fieldconfig object, as seen in the first entry in the example constraint. Note that by default, the path *may* be generated, but it can also be missing. To force the field to be present every time, set `required` to true

Paths may also be a regex. This way you can match multiple paths. Be sure to set `regex` to true in this case. Kubefuzz will throw an error if a path does not exist or a regex doesn't match a single path.

#### Values, regex_values and values_mode

For some fields you may wish to generate more specific values than just type conform ones (like generic string,int etc). This can be done with `values` and `regex_values`. Both are arrays from which KubeFuzz is going to randomly chose to set a value. For `values`, you are free to supply any type, also full fledged objects. `regex_values` is an array of regex strings. Lastly, add `values_mode` to specify the behavior of KubeFuzz if there is an already existing enum at the fields place, possible values are `override` and `add`.

#### Minmax

Minmax is an array with two elements, representing the minimum and maximum number of array elements for arrays.

#### Remove

If set to true, the path is explicitly removed. This can be useful if you want to generate a whole subtree like "\$.spec.containers" but now want to exclude "\$.spec.containers.resources"
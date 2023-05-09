# This generates a json spec for all k8s resources by querying your local kubernetes api on /openapi/v2 to read the swagger 2.0 spec
#
#
# 1. make sure kind is running (use yaml file for kind cluster config)
# 2. 'kubectl proxy' to expose the api to everyone locally
# 3. run this script

openapi2jsonschema --stand-alone http://127.0.0.1:8001/openapi/v2

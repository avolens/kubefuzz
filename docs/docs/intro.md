---
sidebar_position: 1 
---

# Building and Installing 

First clone the repo and cd into it:

```
git clone https://github.com/avolens/kubefuzz
cd kubefuzz
```

Now select a release version from the available git tags.
you can list them by doing


```
git tag
```

and select a tag with

```
git checkout <tagname>
```

To run the latest development build, which might have breaking changes, just stay
on the most recent commit. When running Kubefuzz, you will notice the version is replaced
by "dev".

Now you can now either just build the project by running

`cargo build -r ` or directly install it to your configured cargo binary path by issuing 
`cargo install --path .`

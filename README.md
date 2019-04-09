# cartório

> Serve your container images from a read-only registry.


<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [Usage](#usage)
  - [Docker](#docker)
- [Scope](#scope)
- [Supported image formats](#supported-image-formats)
- [LICENSE](#license)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->


## Usage


The usage of `cartorio` requires only two steps:

1. loading one or more container image(s) from a tarball, then
2. serving those container images to those implementing the [Docker Registry HTTP API v2](https://docs.docker.com/registry/spec/api/).



### Docker

```sh
# save one (or more container images) from a docker 
# daemon to a tarball
docker save one-image another-image > image.tar


# load the image into a format that `cartorio` understands
# and is able to use to serve the contents
cartorio load --tarball=./image.tar


# serve the images
cartorio serve


# pull the image from cartorio
docker pull $MACHINE_IP:5000/one-image
docker pull $MACHINE_IP:5000/another-image
```


### Kubernetes

Being `cartorio` a tool that can serve any amount of container images, the use of `cartorio` with Kubernetes
can fit multiple purposes, more interestingly:

- providing the necessary infratructure images for bootstrapping an airgapped Kubernetes cluster, and
- in a single container, distribute images that can't be retrieved fr



## Scope

`cartorio`'s scope is limited **only** to:

- loading images into its blobstore for serving, and
- serving container images that have been preloaded.



## Supported image formats

- [x] Docker tarball
- [ ] OCI



## LICENSE

Apache License 2.0 - see [LICENSE](./LICENSE).



# serving tarballs as images

`cartorio` is meant to act as a thin layer on top of content to serve that content from consumers of the registry API. 


```

                      registry
                        API
                         |
 ---------.              |
          |              |
 content  +--- cartorio -+-->  docker pull
          |              |
 ---------*              |
                         |
```


Here you can find an answer to "what's the minimum I should implement to serve images to container engines?"


<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [literature](#literature)
- [following the pcap](#following-the-pcap)
  - [a simple image](#a-simple-image)
  - ["sniffing the wire"](#sniffing-the-wire)
    - [checking the version](#checking-the-version)
- [context](#context)
- [topics](#topics)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->


## literature

**ps.: if you don't care about specifications and standards, go straight to [the next section](#following-the-pcap)**

Most of the container-related tech that we use is now in the process of being standardized under the [Open Containers Initiative (OCI)](https://www.opencontainers.org/):

- how to run a container? see [OCI Runtime Spec](https://github.com/opencontainers/runtime-spec)
- how can a container image be described? see [OCI Image Spec](https://github.com/opencontainers/image-spec)
- how could a container image be distributed? see [OCI Distribution Spec](https://github.com/opencontainers/distribution-spec)

For the purpose of describing `cartorio`, here we focus only on the later - the distribution spec - even though pratically, that's pretty much the same as the [Docker Registry HTTP API V2](https://docs.docker.com/registry/spec/api/).



## following the pcap

What happens "on the wire" when you `docker pull $image`?


```

    REGISTRY                    DOCKERD


                |        |
  image1        |   ???  |
  image2    <---+--------+----> docker pull
  ....          |        |
  imageN        |        |
                |        |

```



Capturing the request flow, we can see the entirety of what's necessary for having container images distributed through a registry.

To get started, let's create an image from a `Dockerfile` and see how what happens when we pull it from a registry.



### a simple image

One of the most simple types of images that we could create is one that just adds a file to it.


```dockerfile
# starting with a completely empty layer.
#
FROM scratch              

# add a filke that we have locally
#
ADD ./file.txt /file.txt
```


Build this image, and we can see the layers generated:

```sh
# build the Dockerfile under `./assets`, having the context
# from `assets`, then tag the final layer as `file` (this could
# be something like `concourse/concourse`.
#
docker build --tag file ./assets


docker history file
IMAGE           CREATED BY                           
24cca6f78bbd    ADD ./file.txt /file.txt # buildkit 
```


Having the image there, we can now push it to a registry and inspect the request flow.



### "sniffing the wire"

Looking at the result from capturing the packets from a `docker pull` (i.e., putting ourselves between the registry and the docker daemon), we can see the following flow:

```
CLIENT                                          REGISTRY
(dockerd)                                       (dockerhub... gcr...)

-> GET /v2/
                                                  <- OK


-> GET /v2/file/manifests/latest
                                                  <- manifest


-> GET /v2/file/blobs/sha256:f4f15...
                                                  <- blob content


-> GET /v2/file/blobs/sha256:ffb7f...
                                                  <- blob content
```

What's going on there?



#### checking the version

The first thing `docker` tries to check is what's the version the registry understands:

```
GET /v2/ HTTP/1.1
Host: localhost:5000
User-Agent: docker/18.09.2 ...
Accept-Encoding: gzip
Connection: close

HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
Docker-Distribution-Api-Version: registry/2.0   <-------
Connection: close

{}
```

There's not much to focus on here aside from:

- `Docker-Distribution-Api-Version`: this header *should* be set as clients *may* require this header as a way of verifying if the endpoint setves the API.
- the body *might* be interpreted or not - there the registry implementor is able to tell the client which paths are supported, even though no spec says how that object should be written :shrug:



## context

> The docker registry is a service to manage information about docker images and enable their distribution. 


## topics

- registry version check
- registry manifest
- registry blobs

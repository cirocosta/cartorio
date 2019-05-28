# serving tarballs as images

`cartorio` is meant to act as a thin layer on top of content to serve that content from consumers of the registry API. Here you can get up to speed in terms of knowing how serving content through a registry works.


<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [following the pcap](#following-the-pcap)
  - [a simple image](#a-simple-image)
  - ["sniffing the wire"](#sniffing-the-wire)
    - [checking the version](#checking-the-version)
- [context](#context)
- [topics](#topics)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->


## following the pcap

What happens "on the wire" when you `docker pull $image`?

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

Having the image there, we can now push it to a registry and inspect the reques flow.



### "sniffing the wire"

Looking at the result from capturing the packets from a `docker pull`, we can see the following flow:

```
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




## context

> The docker registry is a service to manage information about docker images and enable their distribution. 


## topics

- registry version check
- registry manifest
- registry blobs

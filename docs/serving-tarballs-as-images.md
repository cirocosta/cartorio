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


Here you can find an answer to *"what's the minimum I should implement to serve images to container engines?"*, or, how does that `REGISTRY API` piece looks like in practice.

ps.: this article assumes that you're familiar with the basic usage of Docker (creating containers, images, etc).


<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [literature](#literature)
- [following the pcap](#following-the-pcap)
  - [a simple image](#a-simple-image)
  - ["sniffing the wire"](#sniffing-the-wire)
    - [checking the version](#checking-the-version)
    - [retrieving the image manifest](#retrieving-the-image-manifest)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->


## literature

**ps.: if you don't care about specifications and standards, go straight to [the next section](#following-the-pcap)**

Most of the container-related tech that we use is now in the process of being standardized under the [Open Containers Initiative (OCI)](https://www.opencontainers.org/):

- how to run a container? see [OCI Runtime Spec][oci-runtime-spec]
- how can a container image be described? see [OCI Image Spec][oci-image-spec]
- how could a container image be distributed? see [OCI Distribution Spec][oci-distribution-spec]

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
- the body *might* be interpreted or not - there the registry implementor is able to tell the client which paths are supported, even though no spec says how that object should be written `¯\_(ツ)_/¯`

With the client having validated that it's interacting with a V2 registry, it can then move forward asking for what it really cares about - the container image.



#### retrieving the image manifest

The manifest is a JSON file that acts as the provider of:

1. configuration that describes metadata about that image, and
2. pointers to where the layers that can be composed to form the filesystem


```

  MANIFEST ---+----> what this container image is about              (metadata)
              |
              +----> where you can go to get the configuration
              |      for the container that you'll create      (runtime config)
              |      to run with the result of this image
              |
              *----> where you can go get the layers to mount on       (layers)
                     top of each other to form the rootfs

```



Here's an example of a manifest of a container image that contains a single layer, no extra metadata, and a runtime configuration file:


```
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
  "config": {
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "size": 1192,
    "digest": "sha256:922f19e5e8f8e734b76618a3e1fe4312c9f07f8d5f83b32c7f33dd9ac38decf7"
  },
  "layers": [
    {
      "mediaType": "application/vnd.docker.image.rootfs.diff.tar",
      "size": 2048,
      "digest": "sha256:e4f8be873d750c0f729a43cac89e1c07a347690c3a8464f35c83109b82b0aa09"
    }
  ]
}
```

**ref: [OCI Image Manifest Specification][oci-image-manifest-spec]**

To retrieve that manifest for a given image and a given tag, the following endpoint exists:

```http
GET /v2/<name>/manifests/<reference>
```

In practice, that turns to the following:


```

               max size: 256; each field adhering to 
               regex [a-z0-9]+(?:[._-][a-z0-9]+)*
           .---.                
           |   |                
           | .-+-----------------> repository name (could be `foo/bar[/...]`
           | | |                                    as well)
           | | |            .----> reference
           | | |            |   
-> GET /v2/file/manifests/latest

```

Where the reference is:

- either the exact digest of a manifest that was pushed, or
- a pointer (alias) to the digest of a manifest that has been pushed.

This can be seen in practice when looking at what `cartorio` does with its internal blobstore:


```
 blobstore
 ├── bucket
 │   └── sha256:cceeccbdfb5
 └── manifests
     └── file
         ├── latest -> blobstore/bucket/sha256:cceeccbdf
         └── sha256:cceeccbdf -> blobstore/bucket/sha256:cceeccbdfb546
```


#### retrieving the image's runtime config

As you might remember, whenever you have a `Dockerfile`, you're able to set runtime configurations:

- what environment variables will be available for the user (`ENV`),
- which command to execute by default (`ENTRYPOINT` and `CMD`),
- which directories to have a volume mounted to (`VOLUME`),
- etc

For example:

```
{ ... "config": {
    ...
    "AttachStderr": false,
    "Tty": false,
    "OpenStdin": false,
    "StdinOnce": false,
    "Env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
    ],
    "Cmd": null,
    ...
    "Labels": null
  }, ...  }

```


All of these configurations live in the runtime configuration file that gets referenced by in the manifest:


```txt
MANIFEST
  {..., "config": {
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "size": 1192,
    "digest": "sha256:922f19e5e8f..."
  }}
```

As the distribution spec assumes that any blob referenced can be fetched from a blobstore in a content-addressable manner, by following that `digest` and knowing the `mediaType`, `docker` can both `fetch`, `verify` and `interpret` such configuration.

The registry API then requires the existence of a blob retrieval endpoint:

```http
GET /v2/<name>/manifests/<reference>
```




[oci-image-manifest-spec]: https://github.com/opencontainers/image-spec/blob/master/manifest.md
[oci-runtime-spec]: https://github.com/opencontainers/runtime-spec
[oci-image-spec]: https://github.com/opencontainers/image-spec
[oci-distribution-spec]: https://github.com/opencontainers/distribution-spec
[docker-image-spec]: https://github.com/moby/moby/blob/master/image/spec/v1.md#image-json-description

# vim: set syntax=dockerfile:

FROM golang AS base

	RUN apt update && apt install -y \
	      apt-transport-https \
	      ca-certificates \
	      git \
	      iptables \
	      curl \
	      gnupg2 \
	      software-properties-common


FROM base AS skopeo

	RUN apt install -y \
		libgpgme-dev \
		libassuan-dev \
		btrfs-progs \
		libdevmapper-dev \
		libostree-dev


	RUN git clone https://github.com/containers/skopeo $GOPATH/src/github.com/containers/skopeo
	WORKDIR $GOPATH/src/github.com/containers/skopeo
	RUN make binary-local && install -m 0755 ./skopeo /usr/local/bin/skopeo



FROM concourse/registry-image-resource AS registry-image-resource


FROM base AS final

	RUN set -x && \
		curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add - && \
		add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/debian $(lsb_release -cs) stable" && \
		apt-get update && apt-get -y install docker-ce

	COPY ./start-docker.sh /usr/local/bin/start-docker
	COPY --from=skopeo /usr/local/bin/skopeo /usr/local/bin/skopeo
	COPY --from=registry-image-resource /opt/resource/ /usr/local/registry-image-resource/bin/

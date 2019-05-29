#!/bin/bash

set -o errexit
set -o nounset
set -o xtrace

readonly TMP_DIR="$(mktemp -d -t cartorio-XXXX)"
readonly CARTORIO_PID_FILE="${TMP_DIR}/cartorio.pid"
readonly MACHINE_IP="$(cd $(dirname $0) && ./get-ip.py)"
readonly CONCOURSE_RESOURCE_URL="https://github.com/concourse/registry-image-resource/releases/download/v0.6.0/registry-image-resource-0.6.0-alpine.tgz"


main () {
	docker-save::populate
	cartorio_serve
	pull_from_cartorio
}

pull_from_cartorio () {
	docker rmi $MACHINE_IP:5000/busybox || true
	docker pull $MACHINE_IP:5000/busybox
}

cartorio_serve () {
	cartorio serve --blobstore=$TMP_DIR &
	echo "$!" > $CARTORIO_PID_FILE
	sleep 1
}

cleanup () {
	local cartorio_pid=$(cat $CARTORIO_PID_FILE)

	if [[ -n $cartorio_pid ]]; then
		kill -s SIGTERM $cartorio_pid
	fi
}


docker-save::populate () {
	local tarball_path=${TMP_DIR}/image.tar

	docker pull busybox
	docker save busybox > $tarball_path
	cartorio load --blobstore=$TMP_DIR --docker-save-tarball=$tarball_path
}

concourse-image-resource::populate () {
	curl -SL $CONCOURSE_RESOURCE_URL | tar xvzf - -C ${TMP_DIR}
}

trap cleanup INT TERM EXIT

main "$@"

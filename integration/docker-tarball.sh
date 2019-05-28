#!/bin/bash

set -o errexit
set -o nounset
set -o xtrace

readonly TMP_DIR="$(mktemp -d -t cartorio-XXXX)"
readonly CARTORIO_PID_FILE="${TMP_DIR}/cartorio.pid"
readonly MACHINE_IP="$(cd $(dirname $0) && ./get-ip.py)"

main () {
	populate_blobstore_and_serve
	pull_from_cartorio
}

pull_from_cartorio () {
	docker pull $MACHINE_IP:5000/busybox
}

populate_blobstore_and_serve () {
	local tarball_path=${TMP_DIR}/image.tar

	docker pull busybox
	docker save busybox > $tarball_path
	cartorio load --blobstore=$TMP_DIR --docker-save-tarball=$tarball_path
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

trap cleanup INT TERM EXIT

main "$@"

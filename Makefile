run-registry:
	docker run \
		--detach \
		--publish 5000:5000 \
		--restart always \
		--name registry \
		registry:2

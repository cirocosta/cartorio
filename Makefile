run-registry:
	cd ./assets/registry && \
		docker-compose up --build -d

capture:
	docker exec registry tshark -d tcp.port==5000,http


test:
	cargo test

install:
	cargo install --path=. --force

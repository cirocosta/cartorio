FROM rust:1.33 AS rust


FROM rust AS base

	RUN rustup target add x86_64-unknown-linux-musl

	WORKDIR /usr/src/myapp
	COPY . .

	RUN cargo build --release --target x86_64-unknown-linux-musl


FROM rust AS test

	WORKDIR /usr/src/myapp
	COPY . .
	RUN cargo test


FROM alpine

	COPY --from=base \
		/usr/src/myapp/target/x86_64-unknown-linux-musl/release/cartorio \
		/usr/local/bin/cartorio

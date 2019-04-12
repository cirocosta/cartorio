FROM rust:1.34 AS rust


FROM rust AS base

	RUN rustup target add x86_64-unknown-linux-musl

	WORKDIR /usr/src/myapp
	COPY . .

	RUN cargo build --release --target x86_64-unknown-linux-musl


FROM rust AS test

	ENV RUST_BACKTRACE=1

	WORKDIR /usr/src/myapp
	COPY . .

	RUN cargo test


FROM alpine

	COPY --from=base \
		/usr/src/myapp/target/x86_64-unknown-linux-musl/release/cartorio \
		/usr/local/bin/cartorio

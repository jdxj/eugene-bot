IMAGE_TAG := jdxj/eugene-bot:$(shell grep '^version' Cargo.toml | awk -F' = ' '{print $$2}' | tr -d '"')

.PHONY: build
build:
	cargo build

.PHONY: build.release
build.release:
	cargo build --release --target=x86_64-unknown-linux-musl

.PHONY: build.image
build.image: build.release
	docker buildx build -t $(IMAGE_TAG) .

.PHONY: push.image
push.image: build.image
	docker push $(IMAGE_TAG)
[private]
@default:
	just --list --unsorted

# build static binary
build:
	@# CARGO_HOME and /tmp/.cargo is used to use local cargo download cache
	docker run --rm -it \
		-v "$(pwd)":/build \
		-v "$HOME/.cargo":/tmp/.cargo \
		-w /build \
		--env=CARGO_HOME=/tmp/.cargo \
		ghcr.io/rust-cross/rust-musl-cross:x86_64-musl \
		cargo build --release \
			--target=x86_64-unknown-linux-musl \
			--config build.rustc-wrapper="''"

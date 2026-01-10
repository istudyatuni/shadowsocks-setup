target := "x86_64-unknown-linux-musl"

[private]
@default:
	just --list --unsorted

# build static binary
build: && pack-release
	@# CARGO_HOME and /tmp/.cargo is used to use local cargo download cache
	docker run --rm -it \
		-v "$(pwd)":/build \
		-v "$HOME/.cargo":/tmp/.cargo \
		-w /build \
		--env=CARGO_HOME=/tmp/.cargo \
		ghcr.io/rust-cross/rust-musl-cross:x86_64-musl \
		cargo build --release \
			--target={{ target }} \
			--config build.rustc-wrapper="''"

[private]
pack-release:
	mv target/{{ target }}/release/sssetup target
	tar caf "target/sssetup-v$(./target/sssetup -V | awk -F ' ' '{ print $2 }').tar.xz" target/sssetup

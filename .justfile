target := "x86_64-unknown-linux-musl"

[private]
@default:
	just --list --unsorted

# build static binary in ci
build-ci: build-static-in-docker && pack-release
	@# fix permissions after building with docker
	sudo chown -R $(whoami) target

# build static binary
build-static: test
	nix build

build-docker: test
	nix build '.#docker'
	docker load -i "$(realpath result)"

test:
	cargo test

extract-changelog file:
	@# about sed: https://askubuntu.com/a/849016
	sed -n "/^## $(just get-build-version ./target/sssetup)/,/^## /p" CHANGELOG.md | grep -v '^## ' > "{{ file }}"

[private]
build-static-in-docker *args: test
	@# CARGO_HOME and /tmp/.cargo is used to use local cargo download cache
	docker run --rm \
		-v "$(pwd)":/build \
		-v "$HOME/.cargo":/tmp/.cargo \
		-w /build \
		--env=CARGO_HOME=/tmp/.cargo \
		clux/muslrust \
		cargo build --release \
			--target={{ target }} \
			--config build.rustc-wrapper="''" \
			{{ args }}

[private]
pack-release:
	mv target/{{ target }}/release/sssetup target | :
	cd target && tar caf "sssetup-v$(just get-build-version ./sssetup).tar.xz" sssetup

[private]
[no-cd]
get-build-version exe:
	"{{exe}}" -V | awk -F ' ' '{ print $2 }'

build-fake-cert domain="localhost": (gen-fake-cert domain) (build-static-in-docker "--features=fake-cert")

gen-fake-cert domain="localhost":
	openssl req \
		-x509 \
		-newkey ec \
		-pkeyopt ec_paramgen_curve:prime256v1 \
		-keyout static/fake.key \
		-out static/fake.crt \
		-days 365 \
		-nodes \
		-subj "/C=US/ST=a/L=a/O=a/CN={{ domain }}"

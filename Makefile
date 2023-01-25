all: install test
.PHONY: all

install:
	cargo install cross
	rustup component add clippy

run:
	cargo run -- data

run-docker:
	make build-linux
	docker build -t vidmerger .
	docker container run -it --rm -v `pwd`/data:/data vidmerger

# ----------------------------------------------------------------

test:
	cargo test -- --exact --nocapture $(name)

test-units:
	cargo test --bins -- --exact --nocapture $(name)

lint:
	cargo clippy

# ----------------------------------------------------------------

build:
	make build-linux
	make build-mac
	make build-win
	make shasum

build-linux:
	@echo 'Building for Linux... 🐧'
	cross build --release --target=x86_64-unknown-linux-musl
	mkdir -p target/release-archives && tar -C target/x86_64-unknown-linux-musl/release -czf target/release-archives/vidmerger-linux.tar.gz vidmerger

build-mac:
	@echo 'Building for MacOS... 🍏'
	cross build --release --target=x86_64-apple-darwin
	mkdir -p target/release-archives && tar -C target/x86_64-apple-darwin/release -czf target/release-archives/vidmerger-mac.tar.gz vidmerger

build-win:
	@echo 'Building for Windows... 🏳️‍🌈'
	cross build --release --target x86_64-pc-windows-gnu
	cd target/x86_64-pc-windows-gnu/release && mv vidmerger.exe ../../../tools

shasum:
	shasum -a 256 target/release-archives/vidmerger*
	shasum -a 256 tools/vidmerger.exe

zip-mac:
	tar -C target/release -czf target/release/vidmerger-mac.tar.gz vidmerger

zip-linux:
	tar -C target/release -czf target/release/vidmerger-linux.tar.gz vidmerger

# ----------------------------------------------------------------

publish-choco:
	choco pack
	Get-ChildItem *.nupkg | ren -NewName vidmerger.nupkg
	choco push vidmerger.nupkg --source https://push.chocolatey.org
	Remove-Item vidmerger.nupkg

publish-dockerhub:
	docker build --no-cache -t vidmerger .
	docker tag vidmerger tgotwig/vidmerger:0.3.0
	docker push tgotwig/vidmerger:0.3.0
	docker tag vidmerger tgotwig/vidmerger
	docker push tgotwig/vidmerger

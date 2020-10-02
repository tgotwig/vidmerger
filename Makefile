all: install prepare run test
.PHONY: all

install:
	cargo install cross
	rustup component add clippy

prepare:
	youtube-dl -o data/1.mp4 -f 22 https://www.youtube.com/watch?v=zGDzdps75ns && cd data && cp 1.mp4 2.mp4
	youtube-dl -o data/1.webm -f 302 https://www.youtube.com/watch?v=zGDzdps75ns && cd data && cp 1.webm 2.webm
	echo Files inside of data: && ls data

run:
	cargo run -- data

run-docker:
	make build-linux
	docker build -t vidmerger .
	docker container run -it --rm -v `pwd`/data:/data -e format=mp4 tgotwig/vidmerger

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
	mkdir -p target/release-archives && cd target/x86_64-pc-windows-gnu/release && mv vidmerger.exe ../../release-archives

shasum:
	shasum -a 256 target/release-archives/vidmerger*

publish-choco:
	cpack
	Get-ChildItem *.nupkg | ren -NewName vidmerger.nupkg
	choco push vidmerger.nupkg --source https://push.chocolatey.org
	Remove-Item vidmerger.nupkg

dockerhub:
	docker build --no-cache -t vidmerger .
	docker tag vidmerger tgotwig/vidmerger:0.1.4
	docker push tgotwig/vidmerger:0.1.4
	docker tag vidmerger tgotwig/vidmerger
	docker push tgotwig/vidmerger

test:
	cargo run --release --verbose -- data -f mp4
	cargo run --release --verbose -- data/ -f mp4
	cd data && \
		cargo run --release --verbose -- . -f mp4
	cargo test

lint:
	cargo clippy

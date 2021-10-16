all: install prepare run test
.PHONY: all

install:
	cargo install cross
	rustup component add clippy

prepare:
	yt-dlp -o data/1.mp4 -f 22 https://www.youtube.com/watch?v=zGDzdps75ns || youtube-dl -o data/1.mp4 -f 22 https://www.youtube.com/watch?v=zGDzdps75ns
	cd data && cp 1.mp4 2.mp4	
	@echo ────────────────────────────
	@echo Files inside of data: && ls data

run:
	cargo run -- data -s 640:480

run-docker:
	make build-linux
	docker build -t vidmerger .
	docker container run -it --rm -v `pwd`/data:/data tgotwig/vidmerger

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

test:
	cargo test -- --test-threads 1

lint:
	cargo clippy

publish-choco:
	cpack
	Get-ChildItem *.nupkg | ren -NewName vidmerger.nupkg
	choco push vidmerger.nupkg --source https://push.chocolatey.org
	Remove-Item vidmerger.nupkg

publish-dockerhub:
	docker build --no-cache -t vidmerger .
	docker tag vidmerger tgotwig/vidmerger:0.1.6
	docker push tgotwig/vidmerger:0.1.6
	docker tag vidmerger tgotwig/vidmerger
	docker push tgotwig/vidmerger

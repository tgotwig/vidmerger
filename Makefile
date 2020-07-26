all: install prepare run test
.PHONY: all

install:
	cargo install cross
	rustup component add clippy

prepare:
	youtube-dl -o data/1.mp4 -f 22 https://www.youtube.com/watch?v=zGDzdps75ns
	cd data && cp 1.mp4 2.mp4
	echo Files inside of data: && ls data

run:
	cargo run -- data/ -f mp4

run-docker:
	make build-linux
	docker build -t vidmerger .
	docker container run -it --rm -v `pwd`/data:/data -e format=mp4 tgotwig/vidmerger

build:
	make build-mac
	make build-linux
	make build-win

build-mac:
	@echo 'Building for MacOS... 🍏'
	cross build --release --target=x86_64-apple-darwin;\
	cd target/x86_64-apple-darwin/release ;\
	mv vid_merger vidmerger ;\
	tar -czf vidmerger-mac.tar.gz vidmerger ;\
	shasum -a 256 vidmerger-mac.tar.gz ;\

	mkdir -p target/tars
	mv target/x86_64-apple-darwin/release/vidmerger-mac.tar.gz target/tars

build-linux:
	@echo 'Building for Linux... 🐧'
	cross build --release --target=x86_64-unknown-linux-musl ;\
	cd target/x86_64-unknown-linux-musl/release ;\
	mv vid_merger vidmerger ;\
	tar -czf vidmerger-linux.tar.gz vidmerger ;\
	shasum -a 256 vidmerger-linux.tar.gz ;\

	mkdir -p target/tars
	mv target/x86_64-unknown-linux-musl/release/vidmerger-linux.tar.gz target/tars

build-win:
	@echo 'Building for Windows... 🏳️‍🌈'
	cross build --release --target x86_64-pc-windows-gnu ;\
	cd target/x86_64-pc-windows-gnu/release ;\
	mv vid_merger.exe vidmerger.exe ;\
	tar -czf vidmerger-win.tar.gz vidmerger.exe ;\
	shasum -a 256 vidmerger-win.tar.gz ;\

	mkdir -p target/tars
	mv target/x86_64-pc-windows-gnu/release/vidmerger-win.tar.gz target/tars

publish-choco:
	choco.exe pack ;\
	mv *.nupkg vidmerger.nupkg ;\
	choco.exe push vidmerger.nupkg --source https://push.chocolatey.org/ ;\
	rm vidmerger.nupkg

dockerhub:
	docker build --no-cache -t vidmerger .
	docker tag vidmerger tgotwig/vidmerger:0.1.2
	docker push tgotwig/vidmerger:0.1.2
	docker tag vidmerger tgotwig/vidmerger
	docker push tgotwig/vidmerger

test:
	cargo run --release --verbose -- data -f mp4
	cargo run --release --verbose -- data/ -f mp4
	cd data && \
		cargo run --release --verbose -- . -f mp4

lint:
	cargo clippy

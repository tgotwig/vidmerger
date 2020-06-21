prepare:
	youtube-dl -o data/1.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'
	youtube-dl -o data/2.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'
	mkdir -p target/tars

run:
	cargo run -- data/ -f mp4

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

	mv target/x86_64-apple-darwin/release/vidmerger-mac.tar.gz target/tars

build-linux:
	@echo 'Building for Linux... 🐧'
	cross build --release --target=x86_64-unknown-linux-musl ;\
	cd target/x86_64-unknown-linux-musl/release ;\
	mv vid_merger vidmerger ;\
	tar -czf vidmerger-linux.tar.gz vidmerger ;\
	shasum -a 256 vidmerger-linux.tar.gz ;\

	mv target/x86_64-unknown-linux-musl/release/vidmerger-linux.tar.gz target/tars

build-win:
	@echo 'Building for Windows... 🏳️‍🌈'
	cross build --release --target x86_64-pc-windows-gnu ;\
	cd target/x86_64-pc-windows-gnu/release ;\
	mv vid_merger.exe vidmerger.exe ;\
	tar -czf vidmerger-win.tar.gz vidmerger.exe ;\
	shasum -a 256 vidmerger-win.tar.gz ;\

	mv target/x86_64-pc-windows-gnu/release/vidmerger-win.tar.gz target/tars

dockerhub:
	docker build -t vidmerger .
	docker tag vidmerger tgotwig/vidmerger:0.1.0
	docker push tgotwig/vidmerger:0.1.0
	docker tag vidmerger tgotwig/vidmerger
	docker push tgotwig/vidmerger

test:
	cargo build --release ;\
	target/release/vid_merger	data/ -f mp4 ;\
	cd data ;\
	../target/release/vid_merger . -f mp4

	make build-linux
	docker build -t vidmerger .
	docker container run -it --rm -v `pwd`/data:/data -e format=mp4 tgotwig/vidmerger

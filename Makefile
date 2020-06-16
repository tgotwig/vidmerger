prepare:
	youtube-dl -o data/1.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'
	youtube-dl -o data/2.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'

run:
	cargo run -- data/ -f mp4

build:
	# mac
	cargo build --release --target=x86_64-apple-darwin;\
	cd target/x86_64-apple-darwin/release ;\
	mv vid_merger vidmerger ;\
	tar -czf vidmerger-mac.tar.gz vidmerger ;\
	shasum -a 256 vidmerger-mac.tar.gz ;\

	# linux
	cargo build --release --target=x86_64-unknown-linux-musl ;\
	cd target/x86_64-unknown-linux-musl/release ;\
	mv vid_merger vidmerger ;\
	tar -czf vidmerger-linux.tar.gz vidmerger ;\
	shasum -a 256 vidmerger-linux.tar.gz ;\

	mkdir -p target/tars ;\
	mv target/x86_64-apple-darwin/release/vidmerger-mac.tar.gz target/tars
	mv target/x86_64-unknown-linux-musl/release/vidmerger-linux.tar.gz target/tars

test:
	cargo build --release ;\
	target/release/vid_merger	data/ -f mp4 ;\
	cd data ;\
	../target/release/vid_merger . -f mp4

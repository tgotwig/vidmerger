run:
	cargo run

build:
	cargo build --release ;\
	cd target/release ;\
	mv vid_merger vidmerger ;\
	tar -czf vidmerger-mac.tar.gz vidmerger ;\
	shasum -a 256 vidmerger-mac.tar.gz

prepare:
	youtube-dl -o data/1.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'
	youtube-dl -o data/2.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'

test:
	cargo run -- data/ -f mp4

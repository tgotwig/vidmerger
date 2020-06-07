run:
	cargo run

build:
	cargo build --release

dl-mp4:
	youtube-dl -o data/1.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'
	youtube-dl -o data/2.mp4 -f 22 'https://www.youtube.com/watch?v=zGDzdps75ns'

test:
	cargo run -- data/ -f mp4

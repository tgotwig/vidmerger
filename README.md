# vidmerger

```fish
↪ exa -lh | awk '{print $2,$7}'
Size
328k 1.mp4
328k 2.mp4
```

```fish
↪ vidmerger . -f mp4

Order of merging 👇

file '1.mp4'
file '2.mp4'

Successfully generated 'output.mp4'! 😆🎞
```

```fish
↪ exa -lh | awk '{print $2,$7}'
Size
328k 1.mp4
328k 2.mp4
664k output.mp4
```

## Installation

[Homebrew](https://brew.sh) (Mac):

```bash
brew tap tgotwig/vidmerger
brew install vidmerger
```

## Run it without installation

[Docker](https://hub.docker.com/repository/docker/tgotwig/vidmerger/general) (Linux):

```bash
docker container run -it --rm -v <PATH-TO-VIDS>:/data -e format=<FORMAT> tgotwig/vidmerger
```

# vidmerger

```bash
↪ ls -l
total 1296
-rw-r--r--  1 tgotwig  staff  328832 Jun  7 20:19 1.mp4
-rw-r--r--  1 tgotwig  staff  328832 Jun  7 20:19 2.mp4
```

```bash
↪ vidmerger . -f mp4
Successfully generated 'output.mp4'! 😆🎞
```

```bash
↪ ls -l
total 2608
-rw-r--r--  1 tgotwig  staff  328832 Jun  7 20:19 1.mp4
-rw-r--r--  1 tgotwig  staff  328832 Jun  7 20:19 2.mp4
-rw-r--r--  1 tgotwig  staff      25 Jun 13 22:14 input.txt
-rw-r--r--  1 tgotwig  staff  664998 Jun 13 22:14 output.mp4
```

```bash
↪ cat input.txt # shows the merge-order 🤠
file '1.mp4'
file '2.mp4'
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

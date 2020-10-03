<p align="center"><a href="https://github.com/nastyox/Rando.js#nastyox"><img src="img/fusion.gif" alt="fusion gif" height="128"/></a></p>

<h1 align="center">Vidmerger</h1>
<p align="center">A wrapper around ffmpeg which simplifies merging of multiple videos.</p>
<br>
<p align="center"><img src="img/demo.gif" alt="fusion gif"/></p>

## 🙉 What is this exactly?

Vidmerger is a command-line-tool which uses `ffmpeg` to merge multiple video-files with the same file-extension together into one file called `output.<format>`. It also includes a usage help 🤗

```shell
USAGE:
    vidmerger [OPTIONS] <DIR>

ARGS:
    <DIR>    Sets the directory to use

FLAGS:
    -h, --help       Prints help information
    -p, --preview    Prints previews of the merge-orders without merging them
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>   Specifies which formats should be merged individually,
                            the default is 👉 avchd,avi,flv,mkv,mov,mp4,webm,wmv
```

## ✨ Installing / Getting started

You can install it on all the three major operating systems 🤗

[Homebrew 🍺](https://github.com/TGotwig/homebrew-vidmerger/blob/master/vidmerger.rb) (Mac 🍏):

```bash
brew install ffmpeg # prerequisite
brew tap tgotwig/vidmerger
brew install vidmerger
```

---

[Homebrew 🍺](https://github.com/TGotwig/homebrew-linux-vidmerger/blob/master/vidmerger.rb) (Linux 🐧):

```bash
sudo snap install ffmpeg --edge # prerequisite
brew tap tgotwig/linux-vidmerger
brew install vidmerger
```

---

[Chocolatey 🍫](https://chocolatey.org/packages/vidmerger) (Windows 🏳️‍🌈)

```powershell
choco install ffmpeg # prerequisite
choco install vidmerger
```

---

### ⭐️ Using Vidmerger ⭐️

First of all lets see what we have:

```fish
↪ exa -lh | awk '{print $2,$7}'
Size
328k 1.mp4
328k 2.mp4
```

Just run it like that:

```fish
↪ vidmerger .

Order of merging 👇

file '1.mp4'
file '2.mp4'

Calling: 'ffmpeg -y -f concat -i data/list.txt -c copy data/output.mp4' 🚀

...

Successfully generated 'output.mp4'! 😆🎞
```

per default it goes through the filename extensions `avchd,avi,flv,mkv,mov,mp4,webm,wmv` and merges all mp4 files to out.mp4, all webm files to out.webm, and so on 😊 When everything went smooth, it should look like this:

```fish
↪ exa -lh | awk '{print $2,$7}'
Size
328k 1.mp4
328k 2.mp4
664k output.mp4
```

There it is! 🎉🎊🥳📺

## 🐳 Run it without installing / Getting started

You can also use Docker to run vidmerger without installing anything except Docker, hosted on [Dockerhub](https://hub.docker.com/r/tgotwig/vidmerger), you can use it like so:

```bash
docker container run -it --rm -v <PATH-TO-VIDS>:/data tgotwig/vidmerger
```

## ⚙️ Developing

### Built With

Some crates inside of [Cargo.toml](Cargo.toml) under `dependencies`.

### Prerequisites

- [Rust 2018+](https://www.rust-lang.org/tools/install)
- [youtube-dl](http://ytdl-org.github.io/youtube-dl/download.html)
- [ffmpeg](https://ffmpeg.org/download.html)

### Setting up Dev

Once you are done with installing the prerequisites, you should run `make` (or [nmake](https://docs.microsoft.com/en-us/cpp/build/reference/nmake-reference?view=vs-2019) for Windows) to see if everything runs smooth:

```shell
git clone https://github.com/TGotwig/vidmerger.git
cd vidmerger
make
```

### Building

Run `make build` to build for Mac, Linux and Windows. You can find the compressed files for Github under `target/tars`, and the uncompressed files besides `target/tars`.

### Deploying / Publishing

Take care that the versions in the following files increases:

- [Cargo.toml](Cargo.toml)
- [Makefile](Makefile)
- [src/cli.yaml](src/cli.yaml)
- [vidmerger.nuspec](vidmerger.nuspec)
- [README.md](README.md)

## 📦 Versioning

We use [SemVer](http://semver.org/) for versioning.

## 🧪 Tests

There are some basic shell-tests, which you can run by `make test`.

## 🖼 Style guide

We are using [rust-clippy](rust-clippy) and [rls-vscode](https://github.com/rust-lang/rls-vscode) ✨

## 📜 Licensing

[MIT License with “Commons Clause”](LICENSE)

<!-- https://github.com/elsewhencode/project-guidelines/blob/master/README.sample.md -->

<p align="center"><a><img src="img/merge.gif" alt="img/merge.gif"/></a></p>

<h1 align="center">Vidmerger</h1>
<p align="center">A wrapper around FFmpeg which simplifies merging of multiple videos.</p>

<p align="center"><img src="img/demo.svg" alt="fusion gif"/></p>

```mermaid
%%{init: {'themeVariables': { 'mainBkg': 'white', 'nodeBorder': 'black' }}}%%
graph LR;
Video_A-->Vidmerger;
Video_B-->Vidmerger;
Vidmerger-->FPS_Changer;
FPS_Changer-->Merger;
Merger-->Chapterer;
Chapterer-->Video_A+B;

Vidmerger-->Merger;
Merger-->Video_A+B;
```

| Feature     | Description                                                                                                                                                                                                                                     |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Selector    | Iterates through [this list of file endings](src/main.rs#L34), selects all files which matches with the current file ending except ones which start with a dot. The list can be overwritten by `--format` or `-f`, example: `--format mp4,mkv`. |
| FPS_Changer | After detecting not matching fps values, it scales all the higher fps videos down to the lowest detecting fps value. Can be skipped by `--skip-fps-changer`. The desired fps value can be set by `--fps`, example: `--fps 23.976`.              |
| Chapterer   | After the merge job is done, it creates the same file but with chapters in it, the title is all in between the first dash till the file extension, example: `Video_A - Chapter 1.mp4`. Can be skipped by `--skip-chapterer`.                    |

## 🙉 What is this exactly?

Vidmerger is a command-line-tool which uses **ffmpeg** to merge multiple video-files with the same file-extension into one file, for example running `vidmerger .` on mp4 files would create a merged video called `output.mp4` 🐣

Here is the usage help of vidmerger 🤗

```shell
A wrapper around ffmpeg which simlifies merging multiple videos 🎞  Everything in between the first
`-` till the fill extension of the input files will be used as chapter titles.

USAGE:
    vidmerger [OPTIONS] <TARGET_DIR>

ARGS:
    <TARGET_DIR>    Sets the input file to use

OPTIONS:
    -f, --format <format>     Specifies which formats should be merged individually, the default is
                              👉 3g2,3gp,aac,ac3,alac,amr,ape,au,avi,awb,dts,f4a,f4b,f4p,f4v,flac,flv,m4a,m4b,m4p,m4r,m4v,mkv,mov,mp2,mp3,mp4,mpeg,mpg,oga,ogg,ogm,ogv,ogx,opus,pcm,spx,wav,webm,wma,wmv
        --fps <fps>           Generates videos inside a temporary folder with this fps value and
                              merges them
    -h, --help                Print help information
        --shutdown            For doing a shutdown at the end (needs sudo)
        --skip-chapterer      Skips the chapterer
        --skip-fps-changer    Skips the fps changer
    -V, --version             Print version information
```

## ✨ Installing / Getting started

You can install it on all the three major operating systems 🤗

### X64

| Platform  | Packager                                                                                   | Command                                                                                                                                                                                          |
| :-------- | :----------------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🍎 MacOS   | 🍺 [Homwbrew](https://github.com/TGotwig/homebrew-vidmerger/blob/master/vidmerger.rb)       | brew tap tgotwig/vidmerger<br>brew install vidmerger                                                                                                                                             |
| 🐧 Linux   | 🍺 [Homwbrew](https://github.com/TGotwig/homebrew-linux-vidmerger/blob/master/vidmerger.rb) | brew tap tgotwig/linux-vidmerger<br>brew install vidmerger                                                                                                                                       |
| 🐧 Linux   | 🍺 CURL                                                                                     | sudo curl -L https://github.com/TGotwig/vidmerger/releases/latest/download/vidmerger-linux.tar.gz -o /tmp/vidmerger-linux.tar.gz && sudo tar -xzvf /tmp/vidmerger-linux.tar.gz -C /usr/local/bin |
| 🏳️‍🌈 Windows | 🍫 [Chocolatey](https://community.chocolatey.org/packages/vidmerger)                        | choco install ffmpeg # prerequisite<br>choco install vidmerger                                                                                                                                   |

### ARM64

| Platform | Packager                                                                             | Command                                                                                                                                                                                                           |
| :------- | :----------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🍎 MacOS  | 🍺 [Homwbrew](https://github.com/TGotwig/homebrew-vidmerger/blob/master/vidmerger.rb) | brew tap tgotwig/vidmerger<br>brew install vidmerger                                                                                                                                                              |
| 🐧 Linux  | 🍺 CURL                                                                               | sudo curl -L https://github.com/tgotwig/vidmerger/releases/download/0.3.2/vidmerger-linux-arm64.tar.gz -o /tmp/vidmerger-linux-arm64.tar.gz && sudo tar -xzvf /tmp/vidmerger-linux-arm64.tar.gz -C /usr/local/bin |

## 🐳 Run it without installing

You can also use Docker to run vidmerger without installing anything except Docker, hosted on [Dockerhub](https://hub.docker.com/r/tgotwig/vidmerger).

```bash
docker container run -it -v <PATH-TO-YOUR-VIDEOS>:/data tgotwig/vidmerger
```

Examples with Bash:

```bash
docker container run tgotwig/vidmerger --help
docker container run tgotwig/vidmerger --version
docker container run -it -v ./data/call_merger:/data tgotwig/vidmerger
```

## ⚙️ Developing

### Built With

Rust and some listed Crates inside of [Cargo.toml](Cargo.toml) under _dependencies_.

### Prerequisites

- [Rust 2021](https://www.rust-lang.org/tools/install)
- [ffmpeg](https://ffmpeg.org/download.html)
- [task](https://taskfile.dev/#/installation)

### Setting up Dev

Once you are done with installing the prerequisites, run `task`:

```shell
git clone git@github.com:TGotwig/vidmerger.git
cd vidmerger
task
```

If you see anything to improve, just create an [issue](https://github.com/tgotwig/vidmerger/issues) or directly open a [pull request](https://github.com/tgotwig/vidmerger/pulls) 🤗✨

### Building

Run `task build` to build for Mac, Linux and Windows. You can find the compressed Mac & Linux .tar.gz-archives for Github under _target/tars_, the .exe file for Windows under _tools_.

### Deploying / Publishing

#### Automated steps

- Homebrew (MacOS & Linux): Gets automatically deployed by [release.yml](https://github.com/TGotwig/vidmerger/blob/master/.github/workflows/release.yml) after pushing a git tag.
- Chocolatey (Windows): Gets automatically deployed by [release.yml](https://github.com/TGotwig/vidmerger/blob/master/.github/workflows/release.yml) after pushing a git tag.
- Docker: Gets automatically deployed by [release.yml](https://github.com/TGotwig/vidmerger/blob/master/.github/workflows/release.yml) after pushing a git tag.

## 📦 Versioning

We use [SemVer](http://semver.org/) for versioning.

## 🧪 Tests

- For major tests: `task test` (requires `ffmpeg` to be installed)
- For linting tests: `task lint`

## 🌟 Style guide

- [rust-clippy](rust-clippy)
- [rls-vscode](https://github.com/rust-lang/rls-vscode)
- [conventionalcommits](https://www.conventionalcommits.org/en/v1.0.0)

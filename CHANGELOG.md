# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Cropper** removes black bars before the merge (#50). Skips files whose quick check finds no bars. When the full check runs, it crops if 60% of sampled frames agree. Keeps the source bitrate, and HEVC stays HEVC on every OS. Can be skipped by `--skip-cropper`.
- The "Removing black bars" step shows the same 0–100% progress bar as the merge.
- GPU video encoding for merging (#46). macOS uses VideoToolbox. On Linux and Windows, the first working GPU encoder (NVIDIA NVENC, Intel QSV, or AMD AMF) is used. If none works, the CPU encoder `libx264` is used.

### Fixed

- On macOS, the black bar crop decodes with VideoToolbox, which cuts CPU load and heat. On a 4K film test, CPU time fell from about 349 s to 52 s.
- Inputs whose video size differs from the most common size are scaled and padded to it before the merge, so cropped and uncropped files can be merged together.
- Cropping no longer re-encodes the other inputs before the merge. Inputs are assumed to share one video codec (all H.264 or all HEVC).
- Each input is now checked before the merge starts. If every input fits the output, they are copied as they are. If any input does not fit, all inputs are converted so their codecs match. MP3 output keeps MP3 audio (#102).
- Chapter start times no longer drift earlier with each episode (#48). Chapter times are now stored in milliseconds, and each boundary is calculated from the running total of clip durations instead of whole seconds per clip.
- Merging `.mov` inputs with a `tmcd` timecode stream no longer fails (#53). The timecode track is dropped with `-write_tmcd 0`, and if the merge still fails, each input is converted to MP4 and the merge is retried (#100).
- Merging into MP4-family outputs no longer fails when an input has `pcm_alaw` audio (#17). The audio is re-encoded to AAC (`-c:a aac`) during the merge, because MP4 containers cannot store `pcm_alaw`.

## 🎉 [0.4.1] - 2026-10-15

### Added

- A progress bar from 0% to 100% is shown during the merge and FPS adjustment steps (#1). It uses FFmpeg's progress output and is hidden in verbose mode.

### Changed

- `--help` prints a diagram.

### Fixed

- Preserve filename order when merging videos after FPS conversion (#91).
- Files with a single quote (`'`) in their name no longer cause an error (#69). FFmpeg and ffprobe are now started with direct argument passing instead of through a shell.
- Files are sorted naturally, so `2` comes before `10` when numbers are not zero-padded (#41).

## 🎉 [0.4.0] - 2024-04-20

### Added

- `--verbose` which prints detailed logs.

### Changed

- Chapters will now be included during the merge phase, seperate chapterer phase gets removed.
- It will not wait anymore for a few seconds at the beginning, instead it will print the merge order and wait for you to press ENTER or ESC to continue or not.
- Videos with changed fps from fps_changer will be stored in <TMP_DIR>/fps_changer.
- Selector now case-insensitive, so it selects mp4 and MP4 etc.

### Removed

- Remove `--skip-wait`

### Fixed

- Fix wrong success message after merge failed.

## 🎉 [0.3.2] - 2024-01-14

### Added

- **Chapterer** which creates `output.*` with chapters in it, everything in between the first `-` till the fill extension of the input files will be used as chapter titles 📖. Can be skipped by `--skip-chapterer`.

### Fixed

- Keep subtitles with merger
- Fix --help and others cmds if ffmpeg is not available
- Fix filter function which filtered out files with multiple dots in the filename

## 🎉 [0.3.1] - 2023-01-26

### Fixed

- Print files from FPS Changer in correct order
- Merge files in correct order
- Fix wrong fps detection for floating point numbers

## 🎉 [0.3.0] - 2023-01-25

### Added

- `--skip-wait` which skips the wait time for reading
- **FPS Changer** which detects different fps values and scales down to the lowest number, or an fps value specified via `--fps`, this feature can also be skipped via `--skip-fps-changer`. None of the original videos will be deleted, instead it generates new ones from the originals in an temporary folder and merges with those

### Changed

- Set all `ffmpeg` supported video and audio formats as default
- Make binaries tiny: [johnthagen/min-sized-rust](https://github.com/johnthagen/min-sized-rust)
- Make logs short and consistent
- Panic with error message from ffmpeg if something goes wrong

### Removed

- Remove `--preview`
- Remove `--scale`

### Fixed

- Wraps ffmpeg arguments in quotes to avoid spacing issues
- Works now with folder names which starts with a dot

## 🎉 [0.2.0] - 2022-05-09

### Changed

- Files like `list.txt` and scaled videos will be created inside a temporary folder where the endpoint looks like `8EbQrP3j`:
  - MacOS: `/var/folders/q9/lgznjs3170b27wn5k9jd54g80000gn/T/<8-RAND-CHARS>`
  - Linux: `/tmp/<8-RAND-CHARS>`
  - Windows: `~/AppData/Local/Temp/<8-RAND-CHARS>`
- Video files starting with a `.` will be ignored (can be the case when dealing with network volumes)
- Append the following message to the success message: `(it can still be broken 🙈)`

### Fixed

- Set `-safe 0` to get rid of `[concat @ 0x55c6fb1e7600] Unsafe file name`

## 🎉 [0.2.0] - 2021-09-05

### Added

- Added `--shutdown` flag for doing a system shutdown after script execution

### Changed

- Improved `--preview`

## 🎉 [0.1.5] - 2021-07-04

### Added

- Scaling videos with the `--scale` / `-s` flag and a value like `320:240` before merging

### Changed

- Wait for `3 seconds` before merging after showing the merge order for having time to read
- Improved logging

### Fixed

- Small fix for the printed ffmpeg command

## 🎉 [0.1.4] - 2020-10-02

### Added

- The `--preview` flag was added

### Changed

- Vidmerger can now run without the `--format` flag

## 🎉 [0.1.2] - 2020-07-29

### Fixed

- Fix issues with backslash-paths on Windows

## 🎉 [0.1.1] - 2020-06-27

### Added

- Show merge-order before merging

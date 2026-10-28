use crate::cli::Cli;
use clap::ArgMatches;
use crossterm::{
  event::{read, Event, KeyCode},
  terminal::{disable_raw_mode, enable_raw_mode},
};
use lazy_static::lazy_static;
use nanoid::nanoid;
use std::cmp::Ordering;
use std::env::temp_dir;
use std::fs::{self, canonicalize, File};
use std::io::{self, Result, Write};
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::process;
use std::process::exit;
use std::str::Chars;

lazy_static! {
  static ref MATCHES: ArgMatches = Cli::init().get_matches();
  static ref VERBOSE: bool = MATCHES.get_flag("verbose");
}

/// Whether `--verbose` was given. Under `cargo test` it is always false: the process
/// arguments belong to the test harness, and parsing them would exit the process.
pub fn verbose() -> bool {
  if cfg!(test) {
    false
  } else {
    *VERBOSE
  }
}

/// Exits the program with an error message if `ffmpeg` is not on the PATH.
pub fn exit_when_ffmpeg_not_available() {
  if which::which("ffmpeg").is_err() {
    eprintln!("❌ ffmpeg is not available. Please install it first.");
    exit(1);
  }
}

/// Deletes `path` if it exists. Does nothing and returns `Ok` when it is missing.
pub fn remove_file(path: &Path) -> Result<()> {
  if Path::new(path).exists() {
    if verbose() {
      print!(
        "🗑️  Removing old data: `{}`",
        path.file_name().unwrap().to_string_lossy()
      );
    }
    fs::remove_file(path)?;
  }
  Ok(())
}

/// Lists the paths of every entry in a folder. Returns `Err` if the folder cannot be read.
pub fn read_dir(input_vids_path: &Path) -> Result<Vec<PathBuf>> {
  fs::read_dir(input_vids_path)?
    .map(|res| res.map(|e| e.path()))
    .collect::<Result<Vec<_>>>()
}

/// Creates a new folder with a random name in the system temp directory and returns its path.
pub fn create_tmp_dir() -> PathBuf {
  let dir = temp_dir().join(nanoid!(8));
  fs::create_dir(&dir).unwrap();
  dir
}

/// Creates the `fps_changer` subfolder inside `base_path` and returns its path.
pub fn create_dir_for_fps_changer(base_path: &Path) -> io::Result<PathBuf> {
  let mut new_path = base_path.to_path_buf();
  new_path.push("fps_changer");
  fs::create_dir_all(&new_path)?;
  Ok(new_path)
}

/// Writes `buf` to the file at `path`, replacing any existing content. Returns `path`.
pub fn create(path: &PathBuf, buf: String) -> &PathBuf {
  File::create(path)
    .unwrap()
    .write_all(buf.as_bytes())
    .unwrap();
  path
}

/// Compares two strings "naturally": runs of digits are compared by their
/// numeric value, so "2" sorts before "10". Other characters compare normally.
/// Ties (e.g. "01" vs "1") are broken by the plain string order.
pub fn natural_cmp(left: &str, right: &str) -> Ordering {
  let mut left_chars = left.chars().peekable();
  let mut right_chars = right.chars().peekable();

  loop {
    match (left_chars.peek(), right_chars.peek()) {
      (None, None) => return left.cmp(right),
      (None, Some(_)) => return Ordering::Less,
      (Some(_), None) => return Ordering::Greater,
      (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit() => {
        let left_digits = take_digits(&mut left_chars);
        let right_digits = take_digits(&mut right_chars);
        let left_number = left_digits.trim_start_matches('0');
        let right_number = right_digits.trim_start_matches('0');
        let by_value = left_number
          .len()
          .cmp(&right_number.len())
          .then_with(|| left_number.cmp(right_number));
        if by_value != Ordering::Equal {
          return by_value;
        }
      }
      (Some(a), Some(b)) => match a.cmp(b) {
        Ordering::Equal => {
          left_chars.next();
          right_chars.next();
        }
        other => return other,
      },
    }
  }
}

/// Consumes and returns the run of digits at the front of `chars`.
fn take_digits(chars: &mut Peekable<Chars>) -> String {
  let mut digits = String::new();
  while let Some(c) = chars.peek().filter(|c| c.is_ascii_digit()) {
    digits.push(*c);
    chars.next();
  }
  digits
}

/// Returns the canonical (absolute) paths as strings, sorted by file name in natural order.
pub fn path_bufs_to_sorted_strings(path_bufs: &[PathBuf]) -> Vec<String> {
  let mut sorted_path_bufs = path_bufs.to_vec();
  // FPS-adjusted clips live in a different directory, so sort by filename rather than full path.
  // Natural order keeps "2" before "10" when files are numbered without zero-padding.
  sorted_path_bufs.sort_by(|left, right| {
    natural_cmp(
      &left.file_name().unwrap_or_default().to_string_lossy(),
      &right.file_name().unwrap_or_default().to_string_lossy(),
    )
  });

  sorted_path_bufs
    .iter()
    .map(|path_buf| {
      canonicalize(path_buf.to_str().unwrap())
        .unwrap()
        .display()
        .to_string()
    })
    .collect()
}

/// Waits for the user to press ENTER (continue) or ESC (exit the program).
pub fn wait_for_enter_or_esc_key() {
  println!("❓ Press ENTER to continue or ESC to cancel...");
  enable_raw_mode().unwrap();
  loop {
    if let Event::Key(key_event) = read().unwrap() {
      match key_event.code {
        KeyCode::Enter => {
          break;
        }
        KeyCode::Esc => {
          disable_raw_mode().unwrap();
          process::exit(0);
        }
        _ => {}
      }
    }
  }
  disable_raw_mode().unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn natural_cmp_sorts_numbers_by_value() {
    assert_eq!(natural_cmp("2", "10"), Ordering::Less);
    assert_eq!(natural_cmp("10", "2"), Ordering::Greater);
    assert_eq!(natural_cmp("file2.mp4", "file10.mp4"), Ordering::Less);
  }

  #[test]
  fn natural_cmp_handles_leading_zeros() {
    assert_eq!(natural_cmp("01", "2"), Ordering::Less);
    assert_eq!(natural_cmp("007", "10"), Ordering::Less);
    assert_eq!(natural_cmp("100", "99"), Ordering::Greater);
  }

  #[test]
  fn natural_cmp_breaks_ties_by_string_order() {
    assert_eq!(natural_cmp("01", "1"), Ordering::Less);
    assert_eq!(natural_cmp("a", "a"), Ordering::Equal);
  }

  #[test]
  fn natural_cmp_compares_text_normally() {
    assert_eq!(natural_cmp("a-clip.mp4", "b-clip.mp4"), Ordering::Less);
    assert_eq!(natural_cmp("abc", "abcd"), Ordering::Less);
    assert_eq!(natural_cmp("", "a"), Ordering::Less);
  }

  #[test]
  fn sorted_names_put_2_before_10() {
    let mut names = vec!["10.mp4", "1.mp4", "3.mp4", "2.mp4", "11.mp4"];
    names.sort_by(|a, b| natural_cmp(a, b));
    assert_eq!(names, ["1.mp4", "2.mp4", "3.mp4", "10.mp4", "11.mp4"]);
  }
}

/// Integration tests: these use the real file system in a temp folder.
#[cfg(test)]
mod test_io_integration {
  use super::*;

  /// A temp folder that is removed when the test ends, even if an assert fails.
  struct TempDir(PathBuf);

  impl TempDir {
    fn new(name: &str) -> Self {
      let dir = temp_dir().join(format!("vidmerger-io-{name}-{}", process::id()));
      let _ = fs::remove_dir_all(&dir);
      fs::create_dir_all(&dir).unwrap();
      TempDir(dir)
    }
  }

  impl Drop for TempDir {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.0);
    }
  }

  #[test]
  fn create_writes_the_content_to_the_file() {
    let dir = TempDir::new("create");
    let path = dir.0.join("list.txt");
    create(&path, "file 'a.mp4'\n".to_string());
    assert_eq!(fs::read_to_string(&path).unwrap(), "file 'a.mp4'\n");
  }

  #[test]
  fn remove_file_deletes_an_existing_file() {
    let dir = TempDir::new("remove");
    let path = dir.0.join("output.mp4");
    fs::write(&path, b"x").unwrap();
    remove_file(&path).unwrap();
    assert!(!path.exists());
  }

  #[test]
  fn remove_file_is_ok_when_the_file_is_missing() {
    let dir = TempDir::new("remove-missing");
    assert!(remove_file(&dir.0.join("nope.mp4")).is_ok());
  }

  #[test]
  fn read_dir_lists_every_entry_in_the_folder() {
    let dir = TempDir::new("read-dir");
    fs::write(dir.0.join("a.mp4"), b"").unwrap();
    fs::write(dir.0.join("b.mp4"), b"").unwrap();
    let mut names: Vec<String> = read_dir(&dir.0)
      .unwrap()
      .iter()
      .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
      .collect();
    names.sort();
    assert_eq!(names, ["a.mp4", "b.mp4"]);
  }

  #[test]
  fn read_dir_fails_on_a_missing_folder() {
    let dir = TempDir::new("read-dir-missing");
    assert!(read_dir(&dir.0.join("missing")).is_err());
  }

  #[test]
  fn create_tmp_dir_makes_a_new_folder_in_the_system_temp_dir() {
    let path = create_tmp_dir();
    assert!(path.is_dir());
    assert!(path.starts_with(temp_dir()));
    let _ = fs::remove_dir_all(&path);
  }

  #[test]
  fn create_dir_for_fps_changer_makes_the_subfolder() {
    let dir = TempDir::new("fps-dir");
    let created = create_dir_for_fps_changer(&dir.0).unwrap();
    assert_eq!(created, dir.0.join("fps_changer"));
    assert!(created.is_dir());
  }

  #[test]
  fn path_bufs_to_sorted_strings_returns_sorted_absolute_paths() {
    let dir = TempDir::new("sorted");
    fs::write(dir.0.join("b.mp4"), b"").unwrap();
    fs::write(dir.0.join("a.mp4"), b"").unwrap();
    let paths = vec![dir.0.join("b.mp4"), dir.0.join("a.mp4")];
    let strings = path_bufs_to_sorted_strings(&paths);
    assert_eq!(strings.len(), 2);
    assert!(strings[0].ends_with("a.mp4"));
    assert!(strings[1].ends_with("b.mp4"));
    assert!(Path::new(&strings[0]).is_absolute());
  }
}

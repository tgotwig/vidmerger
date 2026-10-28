use crate::commanders::_cmd;
use crate::helpers::str_helper::extract_fps_from_ffmpeg_output;
use std::path::PathBuf;

pub fn get_fps(file_to_merge: &PathBuf) -> f32 {
  let output = _cmd::run_ffmpeg_info_command(file_to_merge).unwrap().stderr;
  let output_str = String::from_utf8_lossy(&output);
  extract_fps_from_ffmpeg_output(output_str)
}

/// Integration tests: these call real ffmpeg on a generated clip.
#[cfg(test)]
mod test_fps_integration {
  use super::get_fps;
  use std::path::PathBuf;
  use std::process::Command;

  /// Creates a 1-second video at `fps` frames per second. The file is removed when dropped.
  struct Clip(PathBuf);

  impl Clip {
    fn new(name: &str, fps: u32) -> Self {
      let path =
        std::env::temp_dir().join(format!("vidmerger-fps-{name}-{}.mp4", std::process::id()));
      let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!("testsrc=d=1:s=160x120:r={fps}"))
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
        .arg(&path)
        .status()
        .expect("ffmpeg must be installed");
      assert!(status.success(), "failed to create test clip");
      Clip(path)
    }
  }

  impl Drop for Clip {
    fn drop(&mut self) {
      let _ = std::fs::remove_file(&self.0);
    }
  }

  #[test]
  fn get_fps_reads_25_fps_from_a_real_clip() {
    let clip = Clip::new("25", 25);
    assert_eq!(get_fps(&clip.0), 25.0);
  }

  #[test]
  fn get_fps_reads_30_fps_from_a_real_clip() {
    let clip = Clip::new("30", 30);
    assert_eq!(get_fps(&clip.0), 30.0);
  }
}

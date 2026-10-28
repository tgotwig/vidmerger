use std::io::{stdout, IsTerminal, Write};

const WIDTH: usize = 30;

/// A one-line progress bar, e.g. `[#########.....................] 30% Merging videos`.
/// It redraws the same terminal line. When stdout is not a terminal (e.g. piped output),
/// nothing is drawn.
pub struct ProgressBar {
  animate: bool,
  label: String,
  last_percent: Option<u32>,
}

impl ProgressBar {
  pub fn start(label: &str) -> Self {
    let mut bar = ProgressBar {
      animate: stdout().is_terminal(),
      label: label.to_string(),
      last_percent: None,
    };
    bar.set(0.0);
    bar
  }

  /// Sets the bar to `fraction` of the way done (0.0 to 1.0). Values outside that range are clamped.
  pub fn set(&mut self, fraction: f64) {
    let percent = (fraction.clamp(0.0, 1.0) * 100.0).round() as u32;
    if self.last_percent == Some(percent) {
      return;
    }
    self.last_percent = Some(percent);
    if self.animate {
      let filled = (percent as usize * WIDTH) / 100;
      print!(
        "\r[{}{}] {:>3}% {}",
        "#".repeat(filled),
        ".".repeat(WIDTH - filled),
        percent,
        self.label
      );
      let _ = stdout().flush();
    }
  }

  /// Ends the bar. If `completed`, the bar stays on screen at 100% and the cursor moves to a new line.
  /// Otherwise the line is cleared.
  pub fn finish(&mut self, completed: bool) {
    if !self.animate {
      return;
    }
    if completed {
      self.set(1.0);
      println!();
    } else {
      print!("\r\x1b[2K");
    }
    let _ = stdout().flush();
  }
}

/// Integration test: this draws to the real terminal and takes a few seconds.
#[cfg(test)]
mod test_progress_integration {
  use super::*;
  use std::thread;
  use std::time::Duration;

  #[test]
  fn progress_bar_fills_over_a_few_seconds() {
    // Run with `cargo test progress_bar_fills_over_a_few_seconds -- --nocapture`
    // to watch the bar go from 0% to 100% in a real terminal.
    let mut bar = ProgressBar::start("Merging videos");
    for step in 1..=30 {
      thread::sleep(Duration::from_millis(100));
      bar.set(step as f64 / 30.0);
    }
    bar.finish(true);
  }
}

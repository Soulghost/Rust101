use std::io::Write;

pub struct LogUtil;

impl LogUtil {
    pub fn log_progress(text: &str, progress: f32) {
        print!("\rProgress: {} - {}%", text, progress * 100.0);
        std::io::stdout().flush().unwrap();
    }
}
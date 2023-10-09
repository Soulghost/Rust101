use std::io::Write;

pub struct LogUtil;

impl LogUtil {
    pub fn log_progress(text: &str, progress: f32) {
        print!("\r\x1B[KProgress: {} - {}%", text, (progress * 100.0) as u8);
        std::io::stdout().flush().unwrap();
    }
}
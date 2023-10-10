use std::{fs::File, io::Write, sync::Mutex};

lazy_static::lazy_static! {
    static ref MAX_COLOR: Mutex<f64> = Mutex::new(f64::MIN);
}

use crate::math::vector::Vector3f;

pub type Bitmap2D = Vec<Vec<Vector3f>>;

pub enum RenderTextureSetMode {
    Overwrite,
    Add,
    // Blend
}

pub struct RenderTexture {
    buffer: Bitmap2D,
    width: u32,
    height: u32
}

impl RenderTexture {
    pub fn new(width: u32, height: u32) -> RenderTexture {
        RenderTexture {
            width,
            height,
            buffer: vec![vec![Vector3f::zero(); width as usize]; height as usize]
        }
    }

    pub fn set(&mut self, x: u32, y: u32, color: Vector3f, mode: RenderTextureSetMode) {
        match mode {
            RenderTextureSetMode::Overwrite => {
                self.buffer[y as usize][x as usize] = color;
            }
            RenderTextureSetMode::Add => {
                self.buffer[y as usize][x as usize] += color;
            }
        }
        
    }

    pub fn get_color_attachment(&mut self) -> &mut Bitmap2D {
        &mut self.buffer
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn dump_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let head = format!("P6\n{} {}\n255\n", self.width, self.height);
        file.write_all(head.as_bytes())?;
        for y in 0..self.height {
            for x in 0..self.width {
                let colors = &self.buffer[y as usize][x as usize];
                let buf: [u8; 3] = [
                    self.encode_color_component(colors.x),
                    self.encode_color_component(colors.y),
                    self.encode_color_component(colors.z)
                ];
                file.write(&buf)?;
            }   
        }
        println!("[Texture] max color is {}", *MAX_COLOR.lock().unwrap());
        Ok(())
    }

    fn encode_color_component(&self, c: f64) -> u8 {
        let mut cur = MAX_COLOR.lock().unwrap();
        if c > *cur {
            *cur = c;
        }
        let val = f64::clamp(c, 0.0, 1.0);
        let result = 255.0 * f64::powf(val, 0.6);
        return result as u8;
    }
}
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
}
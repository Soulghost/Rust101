use super::texture::RenderTexture;

pub struct FrameBuffer {
    render_target: RenderTexture
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> FrameBuffer {
        FrameBuffer {
            render_target: RenderTexture::new(width, height)
        }
    }

    pub fn get_render_target(&mut self) -> &mut RenderTexture {
        return &mut self.render_target
    }
}
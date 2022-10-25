use glow::HasContext;
use iced_graphics::image::{TextureStore, TextureStoreEntry};

#[derive(Debug)]
pub struct Textures;

impl Textures {
    pub fn new() -> Self {
        Self
    }
}

impl TextureStore for Textures {
    type Entry = Entry;
    type State<'a> = &'a glow::Context;

    fn upload(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        gl: &mut &glow::Context,
    ) -> Option<Self::Entry> {
        unsafe {
            let texture = gl.create_texture().expect("create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8_ALPHA8 as i32,
                width as i32,
                height as i32,
                0,
                glow::BGRA,
                glow::UNSIGNED_BYTE,
                Some(data),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as _,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            Some(Entry {
                size: (width, height),
                texture,
            })
        }
    }

    fn remove(&mut self, entry: &Entry, gl: &mut &glow::Context) {
        unsafe { gl.delete_texture(entry.texture) }
    }
}

#[derive(Debug)]
pub struct Entry {
    size: (u32, u32),
    pub texture: glow::NativeTexture,
}

impl TextureStoreEntry for Entry {
    fn size(&self) -> (u32, u32) {
        self.size
    }
}

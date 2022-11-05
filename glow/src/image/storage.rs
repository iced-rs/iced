use iced_graphics::image;
use iced_graphics::Size;

use glow::HasContext;

#[derive(Debug, Default)]
pub struct Storage;

impl image::Storage for Storage {
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
                glow::RGBA,
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
                size: Size::new(width, height),
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
    size: Size<u32>,
    pub(super) texture: glow::NativeTexture,
}

impl image::storage::Entry for Entry {
    fn size(&self) -> Size<u32> {
        self.size
    }
}

use crate::program::{self, Shader};
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
#[cfg(feature = "image_rs")]
use std::cell::RefCell;

pub use iced_graphics::triangle::{Mesh2D, Vertex2D};

#[cfg(feature = "image_rs")]
use iced_graphics::image::raster;

#[cfg(feature = "svg")]
use iced_graphics::image::vector;

mod textures;
use textures::{Entry, Textures};

#[derive(Debug)]
pub(crate) struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertex_buffer: <glow::Context as HasContext>::Buffer,
    transform_location: <glow::Context as HasContext>::UniformLocation,
    textures: Textures,
    #[cfg(feature = "image_rs")]
    raster_cache: RefCell<raster::Cache<Textures>>,
    #[cfg(feature = "svg")]
    vector_cache: vector::Cache<Textures>,
}

impl Pipeline {
    pub fn new(
        gl: &glow::Context,
        shader_version: &program::Version,
    ) -> Pipeline {
        let program = unsafe {
            let vertex_shader = Shader::vertex(
                gl,
                shader_version,
                include_str!("shader/common/image.vert"),
            );
            let fragment_shader = Shader::fragment(
                gl,
                shader_version,
                include_str!("shader/common/image.frag"),
            );

            program::create(
                gl,
                &[vertex_shader, fragment_shader],
                &[(0, "i_Position")],
            )
        };

        let transform_location =
            unsafe { gl.get_uniform_location(program, "u_Transform") }
                .expect("Get transform location");

        unsafe {
            gl.use_program(Some(program));

            let transform: [f32; 16] = Transformation::identity().into();
            gl.uniform_matrix_4_f32_slice(
                Some(&transform_location),
                false,
                &transform,
            );

            gl.use_program(None);
        }

        let vertex_buffer =
            unsafe { gl.create_buffer().expect("Create vertex buffer") };
        let vertex_array =
            unsafe { gl.create_vertex_array().expect("Create vertex array") };

        unsafe {
            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));

            let vertices = &[0u8, 0, 1, 0, 0, 1, 1, 1];
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                vertices.len() as i32,
                glow::STATIC_DRAW,
            );
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(vertices),
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                0,
                2,
                glow::UNSIGNED_BYTE,
                false,
                0,
                0,
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
        }

        Pipeline {
            program,
            vertex_array,
            vertex_buffer,
            transform_location,
            textures: Textures::new(),
            #[cfg(feature = "image_rs")]
            raster_cache: RefCell::new(raster::Cache::default()),
            #[cfg(feature = "svg")]
            vector_cache: vector::Cache::default(),
        }
    }

    #[cfg(feature = "image_rs")]
    pub fn dimensions(
        &self,
        handle: &iced_native::image::Handle,
    ) -> (u32, u32) {
        self.raster_cache.borrow_mut().load(handle).dimensions()
    }

    pub fn draw(
        &mut self,
        mut gl: &glow::Context,
        transformation: Transformation,
        _scale_factor: f32,
        images: &[layer::Image],
    ) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
        }

        #[cfg(feature = "image_rs")]
        let mut raster_cache = self.raster_cache.borrow_mut();
        for image in images {
            let (entry, bounds) = match &image {
                #[cfg(feature = "image_rs")]
                layer::Image::Raster { handle, bounds } => (
                    raster_cache.upload(handle, &mut gl, &mut self.textures),
                    bounds,
                ),
                #[cfg(not(feature = "image_rs"))]
                layer::Image::Raster { handle: _, bounds } => (None, bounds),

                #[cfg(feature = "svg")]
                layer::Image::Vector { handle, bounds } => {
                    let size = [bounds.width, bounds.height];
                    (
                        self.vector_cache.upload(
                            handle,
                            size,
                            _scale_factor,
                            &mut gl,
                            &mut self.textures,
                        ),
                        bounds,
                    )
                }

                #[cfg(not(feature = "svg"))]
                layer::Image::Vector { handle: _, bounds } => (None, bounds),
            };

            unsafe {
                if let Some(Entry { texture, .. }) = entry {
                    gl.bind_texture(glow::TEXTURE_2D, Some(*texture))
                } else {
                    continue;
                }

                let translate = Transformation::translate(bounds.x, bounds.y);
                let scale = Transformation::scale(bounds.width, bounds.height);
                let transformation = transformation * translate * scale;
                let matrix: [f32; 16] = transformation.into();
                gl.uniform_matrix_4_f32_slice(
                    Some(&self.transform_location),
                    false,
                    &matrix,
                );

                gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

                gl.bind_texture(glow::TEXTURE_2D, None);
            }
        }

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    pub fn trim_cache(&mut self, mut gl: &glow::Context) {
        #[cfg(feature = "image_rs")]
        self.raster_cache
            .borrow_mut()
            .trim(&mut self.textures, &mut gl);

        #[cfg(feature = "svg")]
        self.vector_cache.trim(&mut self.textures, &mut gl);
    }
}

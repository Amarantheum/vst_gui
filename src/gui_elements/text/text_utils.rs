use gl::types::*;
use glyph_brush::{ab_glyph::*};
use std::{ffi::CString, mem, ptr, str};
use crate::gui_elements::utils::gl::{compile_shader, link_program, gl_err_to_str};
use crate::gl_log_error;

use crate::gui_elements::utils::t_matrix::TMatrix;


// taken from https://github.com/alexheretic/glyph-brush/blob/master/glyph-brush/examples/opengl.rs
pub type Res<T> = Result<T, Box<dyn std::error::Error>>;
/// `[left_top * 3, right_bottom * 2, tex_left_top * 2, tex_right_bottom * 2, color * 4]`
pub type Vertex = [GLfloat; 13];

#[inline]
pub fn to_vertex(
    glyph_brush::GlyphVertex {
        mut tex_coords,
        pixel_coords,
        bounds,
        extra,
    }: glyph_brush::GlyphVertex,
) -> Vertex {
    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    [
        gl_rect.min.x,
        gl_rect.max.y,
        extra.z,
        gl_rect.max.x,
        gl_rect.min.y,
        tex_coords.min.x,
        tex_coords.max.y,
        tex_coords.max.x,
        tex_coords.min.y,
        extra.color[0],
        extra.color[1],
        extra.color[2],
        extra.color[3],
    ]
}

#[rustfmt::skip]
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [f32; 16] {
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);
    [
        2.0 / (right - left), 0.0, 0.0, 0.0,
        0.0, 2.0 / (top - bottom), 0.0, 0.0,
        0.0, 0.0, -2.0 / (far - near), 0.0,
        tx, ty, tz, 1.0,
    ]
}

/// The texture used to cache drawn glyphs
pub struct GlGlyphTexture {
    pub name: GLuint,
}

impl GlGlyphTexture {
    pub fn new((width, height): (u32, u32)) -> Self {
        let mut name = 0;
        unsafe {
            // Create a texture for the glyphs
            // The texture holds 1 byte per pixel as alpha data
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::GenTextures(1, &mut name);
            gl::BindTexture(gl::TEXTURE_2D, name);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as _,
                width as _,
                height as _,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl_log_error!();

            Self { name }
        }
    }

    pub fn clear(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.name);
            gl::ClearTexImage(
                self.name,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                [12_u8].as_ptr() as _,
            );
            gl_log_error!();
        }
    }
}

impl Drop for GlGlyphTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.name);
        }
    }
}

pub struct GlTextPipe {
    shaders: [GLuint; 2],
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    proj_uniform: GLint,
    window_transform_uniform: GLint,
    vertex_count: usize,
    vertex_buffer_len: usize,
}

impl GlTextPipe {
    pub fn new(window_size: (i32, i32), window_transform: &TMatrix) -> Res<Self> {
        let (w, h) = (window_size.0 as f32, window_size.1 as f32);

        let vs = compile_shader(include_str!("shaders/text.vs"), gl::VERTEX_SHADER)?;
        let fs = compile_shader(include_str!("shaders/text.fs"), gl::FRAGMENT_SHADER)?;
        let program = link_program(vs, fs)?;

        let mut vao = 0;
        let mut vbo = 0;
        let window_transform_uniform;
        let proj_uniform = unsafe {
            // Create Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create a Vertex Buffer Object
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Use shader program
            gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0, CString::new("out_color")?.as_ptr());

            // set window_transform uniform
            window_transform_uniform = gl::GetUniformLocation(program, CString::new("window_transform")?.as_ptr());
            if window_transform_uniform < 0 {
                return Err(format!("GetUniformLocation(\"window_transform\") -> {}", window_transform_uniform).into());
            }
            gl::UniformMatrix3fv(window_transform_uniform, 1, 0, window_transform.as_ptr());
            

            // Specify the layout of the vertex data
            let uniform = gl::GetUniformLocation(program, CString::new("proj")?.as_ptr());
            if uniform < 0 {
                return Err(format!("GetUniformLocation(\"proj\") -> {}", uniform).into());
            }
            let transform = ortho(0.0, w, 0.0, h, 1.0, -1.0);
            gl::UniformMatrix4fv(uniform, 1, 0, transform.as_ptr());

            let check:[f32; 9] = [0.0; 9];
            gl::GetUniformfv(program, window_transform_uniform, check.as_ptr() as *mut f32);
            vst_log::log(format!("{:?} and {:?}", check, transform));

            let mut offset = 0;
            for (v_field, float_count) in &[
                ("left_top", 3),
                ("right_bottom", 2),
                ("tex_left_top", 2),
                ("tex_right_bottom", 2),
                ("color", 4),
            ] {
                let attr = gl::GetAttribLocation(program, CString::new(*v_field)?.as_ptr());
                if attr < 0 {
                    return Err(format!("{} GetAttribLocation -> {}", v_field, attr).into());
                }
                gl::VertexAttribPointer(
                    attr as _,
                    *float_count,
                    gl::FLOAT,
                    gl::FALSE as _,
                    mem::size_of::<Vertex>() as _,
                    offset as _,
                );
                gl::EnableVertexAttribArray(attr as _);
                gl::VertexAttribDivisor(attr as _, 1); // Important for use with DrawArraysInstanced

                offset += float_count * 4;
            }

            // Enabled alpha blending
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            // Use srgb for consistency with other examples
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::ClearColor(0.02, 0.02, 0.02, 1.0);
            gl_log_error!();
            uniform
        };

        Ok(Self {
            shaders: [vs, fs],
            program,
            vao,
            vbo,
            proj_uniform,
            window_transform_uniform,
            vertex_count: 0,
            vertex_buffer_len: 0,
        })
    }

    pub fn upload_vertices(&mut self, vertices: &[Vertex]) {
        // Draw new vertices
        self.vertex_count = vertices.len();

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            if self.vertex_buffer_len < self.vertex_count {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertex_count * mem::size_of::<Vertex>()) as GLsizeiptr,
                    vertices.as_ptr() as _,
                    gl::DYNAMIC_DRAW,
                );
                self.vertex_buffer_len = self.vertex_count;
            } else {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (self.vertex_count * mem::size_of::<Vertex>()) as GLsizeiptr,
                    vertices.as_ptr() as _,
                );
            }
            gl_log_error!();
        }
    }

    pub fn update_geometry(&self, window_size: (i32, i32)) {
        let (w, h) = (window_size.0 as f32, window_size.1 as f32);
        let transform = ortho(0.0, w, 0.0, h, 1.0, -1.0);

        unsafe {
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.proj_uniform, 1, 0, transform.as_ptr());
            gl_log_error!();
        }
    }

    pub fn update_window_transform(&self, t: TMatrix) {
        unsafe {
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.window_transform_uniform, 1, 0, t.as_ptr());
            gl_log_error!();
        }
    }

    pub fn load_tmatrix_uniform(&self, t: &TMatrix, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        t.load_as_uniform(self.program, name)
    }

    pub fn draw(&self) {
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            // If implementing this yourself, make sure to set VertexAttribDivisor as well
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.vertex_count as _);
            gl_log_error!();
        }
    }
}

impl Drop for GlTextPipe {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            self.shaders.iter().for_each(|s| gl::DeleteShader(*s));
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
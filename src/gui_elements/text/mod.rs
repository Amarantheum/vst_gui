use glyph_brush::{ab_glyph::*, *};
use text_utils::*;
use crate::gui_elements::utils::t_matrix::TMatrix;

mod text_utils;


pub struct UIText {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    glyph_brush: GlyphBrush<Vertex, Extra, glyph_brush::ab_glyph::FontRef<'static>>,
    text_pipe: GlTextPipe,
    texture: GlGlyphTexture,
    trs: TMatrix,
}

impl UIText {
    pub fn new(text: &str, font_size: f32, color: [f32; 4], position: [f32; 2], font: FontRef<'static>) -> Result<Self, Box<dyn std::error::Error>> {
        let glyph_brush = GlyphBrushBuilder::using_font(font).build();
        //let trs = TMatrix::translation(position[0], position[1]);
        let trs = TMatrix::rotation(-1.0) * TMatrix::scaling(2.0);
        let text_pipe = GlTextPipe::new((640, 360), &trs)?;
        let texture = GlGlyphTexture::new(glyph_brush.texture_dimensions());
        Ok(UIText {
            text: text.to_string(),
            font_size,
            color,
            trs,
            glyph_brush,
            text_pipe,
            texture,
        })
    }

    pub fn render(&mut self, window_size: (i32, i32)) {    
    
        let base_text = Text::new(&self.text).with_scale(self.font_size);
    
        // Queue up all sections of text to be drawn
        self.glyph_brush.queue(
            Section::default()
                .add_text(base_text.with_color(self.color))
                .with_bounds((window_size.0 as f32, window_size.1 as f32)),
        );
    
        // Tell glyph_brush to process the queued text
        let mut brush_action;
        loop {
            brush_action = self.glyph_brush.process_queued(
                |rect, tex_data| unsafe {
                    // Update part of gpu texture with new glyph alpha values
                    gl::BindTexture(gl::TEXTURE_2D, self.texture.name);
                    gl::TexSubImage2D(
                        gl::TEXTURE_2D,
                        0,
                        rect.min[0] as _,
                        rect.min[1] as _,
                        rect.width() as _,
                        rect.height() as _,
                        gl::RED,
                        gl::UNSIGNED_BYTE,
                        tex_data.as_ptr() as _,
                    );
                    //gl_assert_ok!();
                },
                to_vertex,
            );
    
            let max_image_dimension = {
                let mut value = 0;
                unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut value) };
                value as u32
            };
    
            // If the cache texture is too small to fit all the glyphs, resize and try again
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let (new_width, new_height) = if (suggested.0 > max_image_dimension
                        || suggested.1 > max_image_dimension)
                        && (self.glyph_brush.texture_dimensions().0 < max_image_dimension
                            || self.glyph_brush.texture_dimensions().1 < max_image_dimension)
                    {
                        (max_image_dimension, max_image_dimension)
                    } else {
                        suggested
                    };
                    eprint!("\r                            \r");
                    eprintln!("Resizing glyph texture -> {}x{}", new_width, new_height);
    
                    // Recreate texture as a larger size to fit more
                    self.texture = GlGlyphTexture::new((new_width, new_height));
    
                    self.glyph_brush.resize_texture(new_width, new_height);
                }
            }
        }
        // If the text has changed from what was last drawn, upload the new vertices to GPU
        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => self.text_pipe.upload_vertices(&vertices),
            BrushAction::ReDraw => {}
        }
        self.text_pipe.draw();
    }
}
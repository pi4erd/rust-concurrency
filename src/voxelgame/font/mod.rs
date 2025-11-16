// Font rendering

use cgmath::Point2;
use wgpu_text::{BrushBuilder, TextBrush, glyph_brush::{
    Section as TextSection, Text as GlyphText,
    ab_glyph::{FontRef, InvalidFont},
    Color as TextColor
}};
use winit::dpi::PhysicalSize;

pub struct Text {
    pub position: Point2<f32>,
    pub scale: f32,
    pub text: String,
}

impl Text {
    pub fn new(position: Point2<f32>, scale: f32, text: String) -> Self {
        Self { position, scale, text }
    }
}

pub struct TextQueue<'a> {
    brush: TextBrush<FontRef<'a>>,
    queue: Vec<Text>,
}

impl<'a> TextQueue<'a> {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        font: &'a [u8]
    ) -> Result<Self, InvalidFont> {
        let brush = BrushBuilder::using_font_bytes(font)?
            .build(device, config.width, config.height, config.format);

        Ok(Self {
            brush,
            queue: Vec::new(),
        })
    }

    pub fn push_text(&mut self, text: Text) {
        self.queue.push(text);
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, new_size: PhysicalSize<u32>) {
        self.brush.resize_view(new_size.width as f32, new_size.height as f32, queue);
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass
    ) {
        let mut sections: Vec<TextSection> = Vec::new();
        for text in self.queue.iter() {
            sections.push(
                TextSection::default()
                    .add_text(
                        GlyphText::new(&text.text)
                            .with_color([1.0, 1.0, 1.0, 1.0] as TextColor)
                            .with_scale(text.scale)
                    )
                    .with_screen_position(text.position)
            );
        }

        self.brush.queue(
            device,
            queue,
            sections,
        ).expect("Error while queueing text sections");

        self.brush.draw(render_pass);
        self.queue.clear();
    }
}

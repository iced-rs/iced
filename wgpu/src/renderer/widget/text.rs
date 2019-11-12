impl iced_native::text::Renderer for crate::Renderer {
    fn layout(&self, text: &iced_native::Text, limits: &iced_native::layout::Limits) -> iced_native::layout::Node {
        let limits = limits.width(text.width).height(text.height);
        let size = text.size.map(f32::from).unwrap_or(self.style.text_size);
        let bounds = limits.max();

        let section = wgpu_glyph::Section {
            text: &text.content,
            scale: wgpu_glyph::Scale { x: size, y: size },
            bounds: (bounds.width, bounds.height),
            ..Default::default()
        };

        use glyph_brush::GlyphCruncher;
        let (width, height) = if let Some(bounds) =
            self.text_measurements.borrow_mut().glyph_bounds(&section)
        {
            (bounds.width().ceil(), bounds.height().ceil())
        } else {
            (0.0, 0.0)
        };

        let size = limits.resolve(iced_native::Size::new(width, height));

        iced_native::layout::Node::new(size)
    }

    fn draw(&mut self, text: &iced_native::Text, layout: iced_native::Layout<'_>) -> Self::Output {
        (
            crate::Primitive::Text {
                content: text.content.clone(),
                size: text.size.map(f32::from).unwrap_or(self.style.text_size),
                bounds: layout.bounds(),
                color: text.color.unwrap_or(self.style.text_color),
                horizontal_alignment: text.horizontal_alignment,
                vertical_alignment: text.vertical_alignment,
            },
            iced_native::MouseCursor::OutOfBounds,
        )
    }
}

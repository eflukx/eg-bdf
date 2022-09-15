use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline, DecorationColor,
    },
};

use crate::BdfFont;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfTextStyle<'a, C> {
    font: &'a BdfFont<'a>,

    /// Text (foreground) color
    text_color: C,

    /// backgound color, if `None` background pixels are not drawn
    background_color: Option<C>,

    /// Underline decoration color
    underline_color: DecorationColor<C>,

    /// Strike through decoration color
    strikethrough_color: DecorationColor<C>,

    /// Set relative line spacing, default when zero
    /// Use with care, use small values to tune certain fonts. YMMV
    height_adjust: i32,
}

impl<'a, C: PixelColor> BdfTextStyle<'a, C> {
    pub fn new(font: &'a BdfFont<'a>, color: C) -> Self {
        Self {
            font,
            text_color: color,
            background_color: None,
            height_adjust: 0,
            underline_color: DecorationColor::None,
            strikethrough_color: DecorationColor::None,
        }
    }

    pub fn with_bg_color(self, bg_color: C) -> Self {
        Self {
            background_color: Some(bg_color),
            ..self
        }
    }

    pub fn strikethrough(self) -> Self {
        Self {
            strikethrough_color: DecorationColor::TextColor,
            ..self
        }
    }
    pub fn reset_strikethrough(self) -> Self {
        Self {
            strikethrough_color: DecorationColor::None,
            ..self
        }
    }

    pub fn underline(self) -> Self {
        Self {
            underline_color: DecorationColor::TextColor,
            ..self
        }
    }

    pub fn reset_underline(self) -> Self {
        Self {
            underline_color: DecorationColor::None,
            ..self
        }
    }

    pub fn with_height_adjust(self, height_adjust: i32) -> Self {
        Self {
            height_adjust,
            ..self
        }
    }

    pub fn set_height_adjust(&mut self, height_adjust: i32) {
        self.height_adjust = height_adjust;
    }

    pub fn full_height(&self) -> u32 {
        ((self.font.font_ascent + self.font.font_descent) as i32 + self.height_adjust) as u32
    }

    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => -(self.line_height() as i32 - 1),
            Baseline::Middle => -(self.line_height() as i32 - 1) / 2,
            Baseline::Alphabetic => 0,
            Baseline::Bottom => self.font.font_descent as i32,
        }
    }

    fn draw_decorations<T>(
        &self,
        target: &mut T,
        width: u32,
        position: Point,
    ) -> Result<(), T::Error>
    where
        T: DrawTarget<Color = C>,
    {
        if let Some(color) = self.decoration_color_to_color(self.strikethrough_color) {
            let y = Point::new(0, self.baseline_offset(Baseline::Middle) + 1);
            let rect = Rectangle::new(position + y, Size::new(width, 1));
            target.fill_solid(&rect, color)?;
        }

        if let Some(color) = self.decoration_color_to_color(self.underline_color) {
            let y = Point::new(0, self.baseline_offset(Baseline::Alphabetic) + 1);
            let rect = Rectangle::new(position + y, Size::new(width, 1));
            target.fill_solid(&rect, color)?;
        }

        Ok(())
    }

    fn decoration_color_to_color(&self, dc: DecorationColor<C>) -> Option<C> {
        match dc {
            DecorationColor::TextColor => Some(self.text_color),
            DecorationColor::Custom(custom_color) => Some(custom_color),
            DecorationColor::None => None,
        }
    }
}

impl<C: PixelColor> CharacterStyle for BdfTextStyle<'_, C> {
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<Self::Color>) {
        if let Some(color) = text_color {
            self.text_color = color;
        }
    }

    fn set_background_color(&mut self, background_color: Option<Self::Color>) {
        self.background_color = background_color
    }

    fn set_underline_color(&mut self, underline_color: DecorationColor<Self::Color>) {
        self.underline_color = underline_color
    }

    fn set_strikethrough_color(&mut self, strikethrough_color: DecorationColor<Self::Color>) {
        self.strikethrough_color = strikethrough_color
    }
}

impl<C: PixelColor> TextRenderer for BdfTextStyle<'_, C> {
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        mut position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        position -= Point::new(0, self.baseline_offset(baseline));

        for c in text.chars() {
            let glyph = self.font.get_glyph(c);

            glyph.draw(
                position,
                self.text_color,
                self.background_color,
                self.font.data,
                target,
            )?;

            self.draw_decorations(target, glyph.device_width, position)?;

            position.x += glyph.device_width as i32;
        }

        Ok(position)
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        _baseline: Baseline,
        _target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        let string_width = text
            .chars()
            .map(|c| self.font.get_glyph(c).device_width)
            .sum();

        let height = self.line_height() as i32; //+ self.font.font_descent;
        let full_height = height + self.font.font_descent as i32;

        let pos_adj = position - Point::new(0, self.baseline_offset(baseline) + height);
        let size = Size::new(string_width, full_height as u32);

        TextMetrics {
            bounding_box: Rectangle::new(pos_adj, size),
            next_position: position + size.x_axis(),
        }
    }

    fn line_height(&self) -> u32 {
        (self.font.font_ascent as i32 + self.height_adjust).max(0) as u32
    }
}

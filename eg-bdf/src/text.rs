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
    line_spacing: i32,
}

impl<'a, C: PixelColor> BdfTextStyle<'a, C> {
    pub fn new(font: &'a BdfFont<'a>, color: C) -> Self {
        Self {
            font,
            text_color: color,
            background_color: None,
            line_spacing: 0,
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

    pub fn underline(self) -> Self {
        Self {
            underline_color: DecorationColor::TextColor,
            ..self
        }
    }

    pub fn with_line_spacing(self, line_spacing: i32) -> Self {
        Self {
            line_spacing,
            ..self
        }
    }

    pub fn set_line_spacing(&mut self, line_spacing: i32) {
        self.line_spacing = line_spacing;
    }

    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => -(self.line_height() as i32 - 1),
            Baseline::Middle => -(self.line_height() as i32 - 1) / 2,
            Baseline::Bottom => self.font.font_descent as i32,
            Baseline::Alphabetic => 0,
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

        let height = self.line_height() as i32;

        let v_anchor = match baseline {
            Baseline::Top => Point::zero(),
            Baseline::Bottom => Point::new(0, -(height + self.font.font_descent as i32)),
            Baseline::Middle => Point::new(0, -height / 2),
            Baseline::Alphabetic => Point::new(0, -height),
        };

        TextMetrics {
            bounding_box: Rectangle::new(
                position + v_anchor,
                Size::new(string_width, height as u32),
            ),
            next_position: position + Size::new(string_width, 0),
        }
    }

    fn line_height(&self) -> u32 {
        (self.font.font_ascent as i32 + self.line_spacing).max(0) as u32
    }
}

#![no_std]

use embedded_graphics::{
    iterator::raw::RawDataSlice,
    pixelcolor::raw::{LittleEndian, RawU1},
    prelude::*,
    primitives::Rectangle,
};

pub use eg_bdf_macros::include_bdf;
pub mod text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfFont<'a> {
    pub replacement_character: usize,
    pub glyphs: &'a [BdfGlyph],
    pub data: &'a [u8],

    pub pixel_size: u32,
    pub font_ascent: u32,
    pub font_descent: u32,
}

impl<'a> BdfFont<'a> {
    fn get_glyph(&self, c: char) -> &'a BdfGlyph {
        if let Ok(found_idx) = self.glyphs.binary_search_by(|g| g.character.cmp(&c)) {
            &self.glyphs[found_idx]
        } else {
            &self.glyphs[self.replacement_character]
        }

        // We assume sorted glyphs for doing the binary search.. linear
        // self.glyphs
        //     .iter()
        //     .find(|g| g.character == c)
        //     .unwrap_or_else(|| &self.glyphs[self.replacement_character])
        // &self.glyphs[14]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfGlyph {
    pub character: char,
    pub bounding_box: Rectangle,
    pub device_width: u32,
    pub start_index: usize,
}

impl BdfGlyph {
    fn draw<D: DrawTarget>(
        &self,
        position: Point,
        color: D::Color,
        bg_color: Option<D::Color>,
        data: &[u8],
        target: &mut D,
    ) -> Result<(), D::Error> {
        let mut data_iter = RawDataSlice::<RawU1, LittleEndian>::new(data).into_iter();

        if self.start_index > 0 {
            data_iter.nth(self.start_index - 1);
        }
        let zip = self
            .bounding_box
            .translate(position)
            .points()
            .zip(data_iter);

        if let Some(bg_color) = bg_color {
            zip.map(|(p, c)| (p, if c == RawU1::new(1) { color } else { bg_color }))
                .map(|(p, c)| Pixel(p, c))
                .draw(target)
        } else {
            zip.filter(|(_p, c)| *c == RawU1::new(1))
                .map(|(p, _c)| Pixel(p, color))
                .draw(target)
        }
    }
}

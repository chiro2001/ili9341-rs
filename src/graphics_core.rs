use crate::Ili9341;
use embedded_graphics_core::{
    pixelcolor::{
        raw::{RawU16, RawU4},
        Gray4, Rgb565,
    },
    prelude::*,
    primitives::Rectangle,
};

impl<IFACE, RESET> OriginDimensions for Ili9341<IFACE, RESET> {
    fn size(&self) -> Size {
        Size::new(self.width() as u32, self.height() as u32)
    }
}

#[cfg(feature = "gray4")]
const COLOR_HALF_YELLOW: Rgb565 = Rgb565::new(Rgb565::MAX_R / 2, Rgb565::MAX_G / 2, 0);
#[cfg(feature = "gray4")]
const COLOR_HALF_GREEN: Rgb565 = Rgb565::new(0, Rgb565::MAX_G / 2, 0);
#[cfg(feature = "gray4")]
pub const GUI_COLOR_LUT: [Rgb565; 16] = [
    Rgb565::BLACK,               // 0
    Rgb565::CSS_DARK_SLATE_GRAY, // 1
    Rgb565::YELLOW,              // 2
    Rgb565::GREEN,               // 3
    Rgb565::RED,                 // 4
    Rgb565::MAGENTA,             // 5
    Rgb565::CYAN,                // 6
    Rgb565::CSS_LIGHT_GRAY,      // 7
    Rgb565::CSS_PURPLE,          // 8
    Rgb565::CSS_ORANGE_RED,      // 9
    Rgb565::CSS_DARK_RED,        // 10
    COLOR_HALF_YELLOW,           // 11
    COLOR_HALF_GREEN,            // 12
    Rgb565::WHITE,               // 13
    Rgb565::WHITE,               // 14
    Rgb565::WHITE,               // 15
];

#[cfg(feature = "gray4")]
fn convert_to_rgb565_inner(color: Gray4) -> u16 {
    let color = RawU4::from(color).into_inner();
    RawU16::from(GUI_COLOR_LUT[color as usize]).into_inner()
}
#[cfg(not(feature = "gray4"))]
fn convert_to_rgb565_inner(color: Rgb565) -> u16 {
    RawU16::from(color).into_inner()
}

impl<IFACE, RESET> DrawTarget for Ili9341<IFACE, RESET>
where
    IFACE: display_interface::WriteOnlyDataCommand,
{
    type Error = display_interface::DisplayError;

    #[cfg(feature = "gray4")]
    type Color = Gray4;
    #[cfg(not(feature = "gray4"))]
    type Color = Rgn565;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if self.bounding_box().contains(point) {
                let x = point.x as u16;
                let y = point.y as u16;
                let color = convert_to_rgb565_inner(color);
                self.draw_raw_slice(x, y, x, y, &[color])?;
            }
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());

        if let Some(drawable_bottom_right) = drawable_area.bottom_right() {
            let x0 = drawable_area.top_left.x as u16;
            let y0 = drawable_area.top_left.y as u16;
            let x1 = drawable_bottom_right.x as u16;
            let y1 = drawable_bottom_right.y as u16;

            if area == &drawable_area {
                // All pixels are on screen
                self.draw_raw_iter(
                    x0,
                    y0,
                    x1,
                    y1,
                    area.points()
                        .zip(colors)
                        .map(|(_, color)| convert_to_rgb565_inner(color)),
                )
            } else {
                // Some pixels are on screen
                self.draw_raw_iter(
                    x0,
                    y0,
                    x1,
                    y1,
                    area.points()
                        .zip(colors)
                        .filter(|(point, _)| drawable_area.contains(*point))
                        .map(|(_, color)| convert_to_rgb565_inner(color)),
                )
            }
        } else {
            // No pixels are on screen
            Ok(())
        }
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.clear_screen(convert_to_rgb565_inner(color))
    }
}

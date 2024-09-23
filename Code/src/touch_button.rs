
use embedded_graphics::{
    mono_font::{ascii::FONT_6X12, iso_8859_10::FONT_10X20, MonoTextStyle},
    pixelcolor::{self, Rgb565},
    prelude::{PixelColor, Point, RgbColor, Size, WebColors, Dimensions},
    primitives::{
        triangle::StyledPixelsIterator, CornerRadii, Primitive, PrimitiveStyle, Rectangle,
        RoundedRectangle, Triangle,
    },
    text::Text,
    Drawable,
};


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct TouchButton<T> where T : Primitive {

    pub primitive: T,
    pub tag: Text

}

impl <T> TouchButton <T>  where T: Primitive {
    pub const fn new(primitive: T, tag: Text) -> Self {
        Self { primitive, tag }
    }
}


/*
impl Dimensions for TouchButton {
    fn bounding_box(&self) -> Rectangle {
        self.primitive.bounding_box()
    }
}
*/

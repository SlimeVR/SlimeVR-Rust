macro_rules! def_color {
    ($name:ident, $r:literal, $g:literal, $b: literal, $a: literal) => {
        pub const $name: RGBA = RGBA::new($r, $g, $b, $a);
    };
    ($name:ident, $r:literal, $g:literal, $b:literal) => {
        def_color!($name, $r, $g, $b, 255);
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl RGBA {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    def_color!(WHITE, 255, 255, 255);
    def_color!(SILVER, 192, 192, 192);
    def_color!(GRAY, 128, 128, 128);
    def_color!(BLACK, 0, 0, 0);
    def_color!(RED, 255, 0, 0);
    def_color!(MAROON, 128, 0, 0);
    def_color!(YELLOW, 255, 255, 0);
    def_color!(OLIVE, 128, 128, 0);
    def_color!(LIME, 0, 255, 0);
    def_color!(GREEN, 0, 128, 0);
    def_color!(AQUA, 0, 255, 255);
    def_color!(TEAL, 0, 128, 128);
    def_color!(BLUE, 0, 0, 255);
    def_color!(NAVY, 0, 0, 128);
    def_color!(FUCHSIA, 255, 0, 255);
    def_color!(PURPLE, 128, 0, 128);
}

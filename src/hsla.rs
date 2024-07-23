pub const HSLA_ONE: u16 = 1024;

const HSLA_ONE_I: i32 = 1024;
const HSLA_HALF_I: i32 = HSLA_ONE_I / 2;
const HSLA_TWO_I: i32 = HSLA_ONE_I * 2;
const HSLA_THREE_I: i32 = HSLA_ONE_I * 3;
const HSLA_FOUR_I: i32 = HSLA_ONE_I * 4;
const HSLA_SIX_I: i32 = HSLA_ONE_I * 6;
const ONE_U32: u32 = 1024;

/// A color represented in HSLA space
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Hsla {
    pub h: u16,
    pub s: u16,
    pub l: u16,
    pub a: u8,
}

/// Convert from an egui::Color32 to an Hsla color
///
/// ```
/// use tshirt_checker::Hsla;
/// use tshirt_checker::HSLA_ONE;
///
/// # fn main() {
///   let green_rgba = egui::Color32::from_rgba_premultiplied(0,255,0,255);
///   // H: 2   (green hue)
///   // S: 1   (full green color saturation)
///   // L: 1/2
///   // A: 255
///   let expected_green_hsla = Hsla::new(2*HSLA_ONE, HSLA_ONE, HSLA_ONE/2, 255);
///   assert_eq!( expected_green_hsla, (&green_rgba).into());
///
///   let pink_rgba = egui::Color32::from_rgba_premultiplied(255, 128, 128, 255);
///   // H: 0    (red hue)
///   // S: 1    (full color saturation)
///   // L: ~.75 (half way between red and white)
///   // A: 255
///   let expected_pink_hsla = Hsla::new(0, HSLA_ONE, 769, 255);
///   assert_eq!( expected_pink_hsla, Hsla::from(&pink_rgba));
///
///   let grey_rgba = egui::Color32::from_rgba_premultiplied(192,192,192,255);
///   // H: 0   (doesn't really matter because of S)
///   // S: 0   (no color at all)
///   // L: ~.75
///   // A: 255
///   let expected_grey_hsla = Hsla::new(0, 0, 771, 255);
///   assert_eq!( expected_grey_hsla, (&grey_rgba).into());
/// # }
/// ```
///
impl From<&egui::Color32> for Hsla {
    fn from(item: &egui::Color32) -> Self {
        let r: i32 = i32::from(item.r()) * HSLA_ONE_I / 255;
        let g: i32 = i32::from(item.g()) * HSLA_ONE_I / 255;
        let b: i32 = i32::from(item.b()) * HSLA_ONE_I / 255;

        let min: i32 = core::cmp::min(core::cmp::min(r, g), b);
        let max: i32 = core::cmp::max(core::cmp::max(r, g), b);

        let l: i32 = (min + max) / 2;

        if min == max {
            return Hsla {
                h: 0,
                s: 0,
                l: u16::try_from(l).unwrap(),
                a: item.a(),
            };
        }

        let s: i32 = if l <= HSLA_HALF_I {
            ((max - min) * HSLA_ONE_I) / (max + min)
        } else {
            ((max - min) * HSLA_ONE_I) / (HSLA_TWO_I - max - min)
        };

        let ht: i32 = if r == max {
            ((g - b) * HSLA_ONE_I) / (max - min)
        } else if g == max {
            HSLA_TWO_I + ((b - r) * HSLA_ONE_I) / (max - min)
        } else {
            HSLA_FOUR_I + ((r - g) * HSLA_ONE_I) / (max - min)
        };

        let h = (ht + HSLA_SIX_I) % (HSLA_SIX_I);

        Hsla::new(
            u16::try_from(h).unwrap(),
            u16::try_from(s).unwrap(),
            u16::try_from(l).unwrap(),
            item.a(),
        )
    }
}

/// Convert from an egui::Color32 to an Hsla color
///
/// ```
/// use tshirt_checker::Hsla;
/// use tshirt_checker::HSLA_ONE;
///
/// # fn main() {
///   let green_rgba = egui::Color32::from_rgba_premultiplied(0,255,0,255);
///   // H: 2   (green hue)
///   // S: 1   (full green color saturation)
///   // L: 1/2
///   // A: 255
///   let expected_green_hsla = Hsla::new(2*HSLA_ONE, HSLA_ONE, HSLA_ONE/2, 255);
///   assert_eq!( expected_green_hsla, green_rgba.into());
///
///   let pink_rgba = egui::Color32::from_rgba_premultiplied(255, 128, 128, 255);
///   // H: 0    (red hue)
///   // S: 1    (full color saturation)
///   // L: ~.75 (half way between red and white)
///   // A: 255
///   let expected_pink_hsla = Hsla::new(0, HSLA_ONE, 769, 255);
///   assert_eq!( expected_pink_hsla, Hsla::from(pink_rgba));
///
///   let grey_rgba = egui::Color32::from_rgba_premultiplied(192,192,192,255);
///   // H: 0   (doesn't really matter because of S)
///   // S: 0   (no color at all)
///   // L: ~.75
///   // A: 255
///   let expected_grey_hsla = Hsla::new(0, 0, 771, 255);
///   assert_eq!( expected_grey_hsla, grey_rgba.into());
/// # }
/// ```
impl From<egui::Color32> for Hsla {
    fn from(item: egui::Color32) -> Self {
        Self::from(&item)
    }
}

impl From<&Hsla> for egui::Color32 {
    // https://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/

    fn from(val: &Hsla) -> Self {
        if val.s == 0 {
            let grey_u32 = u32::from(val.l) * 255 / ONE_U32;
            let grey_u8 = grey_u32.try_into().unwrap();
            return egui::Color32::from_rgba_premultiplied(grey_u8, grey_u8, grey_u8, val.a);
        }
        let h: i32 = i32::from(val.h);
        let s: i32 = i32::from(val.s);
        let l: i32 = i32::from(val.l);

        let temp1: i32 = if l <= HSLA_HALF_I {
            (l * (HSLA_ONE_I + s)) / HSLA_ONE_I
        } else {
            l + s - ((l * s) / HSLA_ONE_I)
        };
        let temp2: i32 = 2 * l - temp1;

        fn hue_to_rgb_2(t1: i32, t2: i32, harg: i32) -> i32 {
            let h = harg % HSLA_SIX_I;
            if h < HSLA_ONE_I {
                t2 + (t1 - t2) * h / HSLA_ONE_I
            } else if h < HSLA_THREE_I {
                t1
            } else if h < HSLA_FOUR_I {
                t2 + (t1 - t2) * (HSLA_FOUR_I - h) / HSLA_ONE_I
            } else {
                t2
            }
        }

        fn hue_to_rgb(t1: i32, t2: i32, h: i32) -> u8 {
            let tmp = hue_to_rgb_2(t1, t2, h) * 255 / HSLA_ONE_I;
            u8::try_from(tmp).unwrap()
        }

        let r = hue_to_rgb(temp1, temp2, h + HSLA_TWO_I);
        let g = hue_to_rgb(temp1, temp2, h);
        let b = hue_to_rgb(temp1, temp2, h + HSLA_FOUR_I);

        egui::Color32::from_rgba_premultiplied(r, g, b, val.a)
    }
}

impl From<Hsla> for egui::Color32 {
    fn from(val: Hsla) -> Self {
        Self::from(&val)
    }
}

impl Hsla {
    pub fn new(h: u16, s: u16, l: u16, a: u8) -> Self {
        Self { h, s, l, a }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_U16: u16 = 1024;
    const TWO_U16: u16 = ONE_U16 * 2;
    const FOUR_U16: u16 = ONE_U16 * 4;
    const HALF_U16: u16 = ONE_U16 / 2;

    // helper
    fn rgba(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(r, g, b, a)
    }

    fn is_close(expected: &egui::Color32, actual: &egui::Color32) -> bool {
        let rd = ((expected.r() as i32) - (actual.r() as i32)).abs();
        let gd = ((expected.g() as i32) - (actual.g() as i32)).abs();
        let bd = ((expected.b() as i32) - (actual.b() as i32)).abs();
        let ad = ((expected.a() as i32) - (actual.a() as i32)).abs();
        rd <= 1 && gd <= 1 && bd <= 1 && ad == 0
    }

    #[test]
    fn test_primaries() {
        let red_hsla = Hsla {
            h: 0,
            s: ONE_U16,
            l: HALF_U16,
            a: 255,
        };
        assert_eq!(rgba(255, 0, 0, 255), red_hsla.into());
        let green_hsla = Hsla {
            h: TWO_U16,
            s: ONE_U16,
            l: HALF_U16,
            a: 255,
        };
        assert_eq!(rgba(0, 255, 0, 255), green_hsla.into());
        let blue_hsla = Hsla {
            h: FOUR_U16,
            s: ONE_U16,
            l: HALF_U16,
            a: 255,
        };
        assert_eq!(rgba(0, 0, 255, 255), blue_hsla.into());
    }

    #[test]
    fn test_identities() {
        for r in 0..16 {
            for g in 0..16 {
                for b in 0..16 {
                    let original = rgba(r * 17, g * 17, b * 17, 255);
                    let hsla: Hsla = (&original).into();
                    let converted: egui::Color32 = (&hsla).into();
                    assert!(
                        is_close(&original, &converted),
                        "expected = {:?} actual = {:?} hsla = {:?}",
                        original,
                        converted,
                        hsla
                    );
                }
            }
        }
    }
}

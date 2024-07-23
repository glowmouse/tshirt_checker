const ONE_I32: i32 = 256;
const ONE_U16: u16 = 256;

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
///
/// # fn main() {
///   let green_rgba = egui::Color32::from_rgba_premultiplied(0,255,0,255);
///   // H: 512 = 2.0 * 256 (green hue)
///   // S: 256 = 1.0 * 256 (full green color saturation)
///   // L: 128 = 0.5 * 256
///   // A: 255 = 1.0 * 255
///   let expected_green_hsla = Hsla::new(512, 256, 128, 255);
///   assert_eq!( expected_green_hsla, (&green_rgba).into());
///
///   let pink_rgba = egui::Color32::from_rgba_premultiplied(255, 128, 128, 255);
///   // H: 0   = 0.0 * 256 (red hue)
///   // S: 256 = 1.0 * 256 (full color saturation)
///   // L: 192 = 0.5 * 256 (~half way between red and white)
///   // A: 255 = 1.0 * 255
///   let expected_pink_hsla = Hsla::new(0, 256, 192, 255);
///   assert_eq!( expected_pink_hsla, Hsla::from(&pink_rgba));
///
///   let grey_rgba = egui::Color32::from_rgba_premultiplied(192,192,192,255);
///   // H: 0   = 0.0 * 256 (doesn't really matter because of S
///   // S: 0   = 0.0 * 256 (no color at all)
///   // L: 192 = 0.5 * 256 (3/4 between black and white)
///   // A: 255 = 1.0 * 255
///   let expected_grey_hsla = Hsla::new(0, 0, 192, 255);
///   assert_eq!( expected_grey_hsla, (&grey_rgba).into());
/// # }
/// ```
///
impl From<&egui::Color32> for Hsla {
    fn from(item: &egui::Color32) -> Self {
        let r: i32 = i32::from(item.r()) * ONE_I32 / 255;
        let g: i32 = i32::from(item.g()) * ONE_I32 / 255;
        let b: i32 = i32::from(item.b()) * ONE_I32 / 255;

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

        let half: i32 = ONE_I32 / 2;
        let two: i32 = ONE_I32 * 2;
        let four: i32 = ONE_I32 * 4;
        let six: i32 = ONE_I32 * 6;

        let s: i32 = if l <= half {
            ((max - min) * ONE_I32) / (max + min)
        } else {
            ((max - min) * ONE_I32) / (two - max - min)
        };

        let ht: i32 = if r == max {
            ((g - b) * ONE_I32) / (max - min)
        } else if g == max {
            two + ((b - r) * ONE_I32) / (max - min)
        } else {
            four + ((r - g) * ONE_I32) / (max - min)
        };

        let h = (ht + six) % (six);

        std::assert!(h >= 0);

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
///
/// # fn main() {
///   let green_rgba = egui::Color32::from_rgba_premultiplied(0,255,0,255);
///   // H: 512 = 2.0 * 256 (green hue)
///   // S: 256 = 1.0 * 256 (full green color saturation)
///   // L: 128 = 0.5 * 256
///   // A: 255 = 1.0 * 255
///   let expected_green_hsla = Hsla::new(512, 256, 128, 255);
///   assert_eq!( expected_green_hsla, green_rgba.into());
///
///   let pink_rgba = egui::Color32::from_rgba_premultiplied(255, 128, 128, 255);
///   // H: 0   = 0.0 * 256 (red hue)
///   // S: 256 = 1.0 * 256 (full color saturation)
///   // L: 192 = 0.5 * 256 (~half way between red and white)
///   // A: 255 = 1.0 * 255
///   let expected_pink_hsla = Hsla::new(0, 256, 192, 255);
///   assert_eq!( expected_pink_hsla, Hsla::from(pink_rgba));
///
///   let grey_rgba = egui::Color32::from_rgba_premultiplied(192,192,192,255);
///   // H: 0   = 0.0 * 256 (doesn't really matter because of S
///   // S: 0   = 0.0 * 256 (no color at all)
///   // L: 192 = 0.5 * 256 (3/4 between black and white)
///   // A: 255 = 1.0 * 255
///   let expected_grey_hsla = Hsla::new(0, 0, 192, 255);
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
            let grey_u16 = val.l * 255 / ONE_U16;
            let grey_u8 = grey_u16.try_into().unwrap();
            return egui::Color32::from_rgba_premultiplied(grey_u8, grey_u8, grey_u8, val.a);
        }
        let h: i32 = i32::from(val.h);
        let s: i32 = i32::from(val.s);
        let l: i32 = i32::from(val.l);

        let temp1: i32 = if l <= 128 {
            (l * (256 + s)) / 256
        } else {
            l + s - ((l * s) / 256)
        };
        let temp2: i32 = 2 * l - temp1;

        fn hue_to_rgb_2(t1: i32, t2: i32, harg: i32) -> i32 {
            let h = harg % (6 * 256);
            let one: i32 = 256;
            let three: i32 = 256 * 3;
            let four: i32 = 256 * 4;
            if h < one {
                t2 + (t1 - t2) * h / 256
            } else if h < three {
                t1
            } else if h < four {
                t2 + (t1 - t2) * (four - h) / 256
            } else {
                t2
            }
        }

        fn hue_to_rgb(t1: i32, t2: i32, h: i32) -> u8 {
            let tmp = hue_to_rgb_2(t1, t2, h) * 255 / 256;
            u8::try_from(tmp).unwrap()
        }

        let r = hue_to_rgb(temp1, temp2, h + 512);
        let g = hue_to_rgb(temp1, temp2, h);
        let b = hue_to_rgb(temp1, temp2, h + 1024);

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

    // helper
    fn rgba(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(r, g, b, a)
    }

    fn is_close(expected: &egui::Color32, actual: &egui::Color32) -> bool {
        let rd = ((expected.r() as i32) - (actual.r() as i32)).abs();
        let gd = ((expected.g() as i32) - (actual.g() as i32)).abs();
        let bd = ((expected.b() as i32) - (actual.b() as i32)).abs();
        let ad = ((expected.a() as i32) - (actual.a() as i32)).abs();
        rd <= 4 && gd <= 4 && bd <= 4 && ad == 0
    }

    #[test]
    fn test_primaries() {
        let red_hsla = Hsla {
            h: 0,
            s: 256,
            l: 128,
            a: 255,
        };
        assert_eq!(rgba(255, 0, 0, 255), red_hsla.into());
        let green_hsla = Hsla {
            h: 512,
            s: 256,
            l: 128,
            a: 255,
        };
        assert_eq!(rgba(0, 255, 0, 255), green_hsla.into());
        let blue_hsla = Hsla {
            h: 1024,
            s: 256,
            l: 128,
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

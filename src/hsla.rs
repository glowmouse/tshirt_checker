const HSLA_ONE_I: i32 = 1024;
const HSLA_ONE_F: f32 = 1024.0;
const HSLA_HALF_I: i32 = HSLA_ONE_I / 2;
const HSLA_TWO_I: i32 = HSLA_ONE_I * 2;
const HSLA_THREE_I: i32 = HSLA_ONE_I * 3;
const HSLA_FOUR_I: i32 = HSLA_ONE_I * 4;
const HSLA_SIX_I: i32 = HSLA_ONE_I * 6;
const ONE_U32: u32 = 1024;
const ONE_U16: u16 = 1024;

///
/// A color represented in HSLA space
///
/// h, s, and l values are 6:10 fix point values (6 bit integer, 10 bit decimal).
/// a is the alpha value from 0 to 255.
///
/// See <https://en.wikipedia.org/wiki/HSL_and_HSV> for more information.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Hsla {
    pub h: u16,
    pub s: u16,
    pub l: u16,
    pub a: u8,
}

///
/// Transformation between HSLA color spaces
///
struct HslaTransform {
    pub ht: u16,
    pub st: Vec<u16>,
    pub lt: Vec<u16>,
}

/// Convert from an egui::Color32 to an Hsla color
///
/// ```
/// use tshirt_checker::Hsla;
///
/// # fn main() {
/// let green_rgba = egui::Color32::from_rgba_premultiplied(0,255,0,255);
/// // H: 2   (green hue)
/// // S: 1   (full green color saturation)
/// // L: 1/2
/// // A: 1
/// let expected_green_hsla = Hsla::newf(2.0, 1.0, 0.5, 1.0);
/// assert_eq!( expected_green_hsla, (&green_rgba).into());
///
/// let pink_rgba = egui::Color32::from_rgba_premultiplied(255, 128, 128, 255);
/// // H: 0    (red hue)
/// // S: 1    (full color saturation)
/// // L: ~.75 (half way between red and white)
/// // A: 1
/// let expected_pink_hsla = Hsla::newf(0.0, 1.0, 0.5+0.5*(128.0/255.0), 1.0);
/// assert_eq!( expected_pink_hsla, Hsla::from(&pink_rgba));
///
/// let grey_rgba = egui::Color32::from_rgba_premultiplied(192,192,192,255);
/// // H: 0   (doesn't really matter because of S)
/// // S: 0   (no color at all)
/// // L: ~.75
/// // A: 1
/// let expected_grey_hsla = Hsla::newf(0.0, 0.0, 192.0/255.0, 1.0);
/// assert_eq!( expected_grey_hsla, (&grey_rgba).into());
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
///
/// # fn main() {
/// let green_rgba = egui::Color32::from_rgba_premultiplied(0,255,0,255);
/// // H: 2   (green hue)
/// // S: 1   (full green color saturation)
/// // L: 1/2
/// // A: 1
/// let expected_green_hsla = Hsla::newf(2.0, 1.0, 0.5, 1.0);
/// assert_eq!( expected_green_hsla, green_rgba.into());
///
/// let pink_rgba = egui::Color32::from_rgba_premultiplied(255, 128, 128, 255);
/// // H: 0    (red hue)
/// // S: 1    (full color saturation)
/// // L: ~.75 (half way between red and white)
/// // A: 1
/// let expected_pink_hsla = Hsla::newf(0.0, 1.0, 0.5+0.5*(128.0/255.0), 1.0);
/// assert_eq!( expected_pink_hsla, Hsla::from(pink_rgba));
///
/// let grey_rgba = egui::Color32::from_rgba_premultiplied(192,192,192,255);
/// // H: 0   (doesn't really matter because of S)
/// // S: 0   (no color at all)
/// // L: ~.75
/// // A: 1
/// let expected_grey_hsla = Hsla::newf(0.0, 0.0, 192.0/255.0, 1.0);
/// assert_eq!( expected_grey_hsla, grey_rgba.into());
/// # }
/// ```
impl From<egui::Color32> for Hsla {
    fn from(item: egui::Color32) -> Self {
        Self::from(&item)
    }
}

/// Convert from an Hsla color to an egui::Color32
///
/// ```
/// use tshirt_checker::Hsla;
/// use egui::Color32;
///
/// # fn main() {
/// // h = 0 (red), s = 1.0 (full red)
/// let red_hsla = Hsla::newf(0.0, 1.0, 0.5, 1.0 );
/// assert_eq!(Color32::from_rgb(255, 0, 0), red_hsla.into());
///
/// // h = 2 (green), s = 1.0 (full green)
/// let green_hsla = Hsla::newf(2.0, 1.0, 0.5, 1.0 );
/// assert_eq!(Color32::from_rgb(0, 255, 0), green_hsla.into());
///
/// // h = 4 (blue), s = 1.0 (full blue)
/// let blue_hsla = Hsla::newf(4.0, 1.0, 0.5, 1.0 );
/// assert_eq!(Color32::from_rgb(0, 0, 255), blue_hsla.into());
///
/// // h = 1 (yellow), s = 1.0, l=.75 (toward white)
/// let yellow_hsla = Hsla::newf(1.0, 1.0, 0.75, 1.0 );
/// assert_eq!(Color32::from_rgb(255, 255, 127), yellow_hsla.into());
/// # }
/// ```
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

/// Convert from an Hsla color to an egui::Color32
///
impl From<Hsla> for egui::Color32 {
    fn from(val: Hsla) -> Self {
        Self::from(&val)
    }
}

impl Hsla {
    ///
    /// Create an Hsla color from fixed point integer values
    ///
    /// h, s, and l values are 6:10 fix point values (6 bit integer, 10 bit decimal).
    /// a is the alpha value from 0 to 255.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV> for more information.
    ///
    /// ```
    /// use tshirt_checker::Hsla;
    /// # fn main() {
    /// // h=2, s=1, l=.5, full alpha
    /// let green_from_fixp = Hsla::new( 2 << 10, 1 << 10, 1 << 9, 255 );
    /// let green_from_float = Hsla::newf( 2.0, 1.0, 0.5, 1.0 );
    /// assert_eq!( green_from_fixp, green_from_float );
    /// # }
    /// ```
    ///
    pub fn new(h: u16, s: u16, l: u16, a: u8) -> Self {
        Self { h, s, l, a }
    }

    ///
    /// Create an Hsla color from floats
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV> for more information.
    ///
    /// ```
    /// use tshirt_checker::Hsla;
    /// # fn main() {
    /// // h=2, s=1, l=.5, full alpha
    /// let green_from_float = Hsla::newf( 2.0, 1.0, 0.5, 1.0 );
    /// let green_from_fixp= Hsla::new( 2 << 10, 1 << 10, 1 << 9, 255 );
    /// assert_eq!( green_from_fixp, green_from_float );
    /// # }
    /// ```
    ///
    pub fn newf(hf: f32, sf: f32, lf: f32, af: f32) -> Self {
        let h = (hf * HSLA_ONE_F) as u16;
        let s = (sf * HSLA_ONE_F) as u16;
        let l = (lf * HSLA_ONE_F) as u16;
        let a = (af * 255.0) as u8;
        Self { h, s, l, a }
    }

    /// What needs to be added to an HSLA hue to shift from one RGB color to another
    ///
    /// See unit test for more details on usage
    ///
    fn calc_hue_shift(source: egui::Color32, target: egui::Color32) -> u16 {
        let source_hsla: Hsla = source.into();
        let target_hsla: Hsla = target.into();
        let shift = (ONE_U32 * 6 + (target_hsla.h as u32) - (source_hsla.h as u32)) % (ONE_U32 * 6);
        shift.try_into().unwrap()
    }

    /// Calculate a fixed point gamma table that maps source to target.
    ///
    /// In the general case, returns a table of fixed point values that computes
    /// the gamma function
    ///
    /// f(x) = x^gamma
    ///
    /// where 0 <= x <= 1, 0 <= f(x) <= 1, and gamma = ln(target) / ln(source),
    ///
    /// This has the property that target = f(source) (i.e, source becomes target)
    ///
    /// The output is a vector with ONE_U16+1 entries in the range of 0 to ONE_U16.
    /// The ONE_U16 fixed point value is equaivalent to 1.0 in floating point.
    ///
    /// The map allows the code to cleanly adjust values like luminance in an image
    /// i.e.,  we can say "luminance value" .5 in the source image will be luminance
    /// value .25 in the new image.  The values 0 and 1 in the source image remains
    /// values 0 and 1 in the new image.
    ///
    /// The function becomes problematic if the source or target is a 0 or a 1.
    /// ln(0) is infinity, and ln(1) is 0.  For now I'm just returning an identity
    /// function.
    ///
    fn calc_gamma_table(source: f32, target: f32) -> Vec<u16> {
        assert!((0.0..=1.0).contains(&source));
        assert!((0.0..=1.0).contains(&target));

        if target == 0.0 || target == 1.0 || source == 0.0 || source == 1.0 {
            // Return identity on these corner cases
            (0..=ONE_U16).collect()
        } else {
            let gamma = target.ln() / source.ln();
            (0..=ONE_U16)
                .map(|n| (n as f32) / HSLA_ONE_F)
                .map(|n| n.powf(gamma))
                .map(|n| (n * HSLA_ONE_F) as u16)
                .collect()
        }
    }

    /// Generate a data structure that transforms HSLA space from source to target
    ///
    ///
    pub fn calc_hsla_transform(
        source: egui::Color32,
        target: egui::Color32,
    ) -> Box<dyn Fn(&egui::Color32) -> egui::Color32> {
        let source_hsla: Hsla = source.into();
        let target_hsla: Hsla = target.into();

        let ht = Self::calc_hue_shift(source, target);

        let source_s = (source_hsla.s as f32) / HSLA_ONE_F;
        let target_s = (target_hsla.s as f32) / HSLA_ONE_F;
        let st = Hsla::calc_gamma_table(source_s, target_s);

        let source_l = (source_hsla.l as f32) / HSLA_ONE_F;
        let target_l = (target_hsla.l as f32) / HSLA_ONE_F;
        let lt = Hsla::calc_gamma_table(source_l, target_l);

        let transform = HslaTransform { ht, st, lt };

        Box::new(move |input: &egui::Color32| -> egui::Color32 {
            let input_hsla = Hsla::from(input);

            let h = Hsla::hue_shift(input_hsla.h, transform.ht);
            let s = transform.st[input_hsla.s as usize];
            let l = transform.lt[input_hsla.l as usize];
            let a = input_hsla.a;
            let adjusted_hsla = Hsla { h, s, l, a };

            adjusted_hsla.into()
        })
    }

    #[inline(always)]
    fn hue_shift(orig: u16, shift: u16) -> u16 {
        (orig + shift) % (ONE_U16 * 6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_close(expected: &egui::Color32, actual: &egui::Color32) -> bool {
        let rd = ((expected.r() as i32) - (actual.r() as i32)).abs();
        let gd = ((expected.g() as i32) - (actual.g() as i32)).abs();
        let bd = ((expected.b() as i32) - (actual.b() as i32)).abs();
        let ad = ((expected.a() as i32) - (actual.a() as i32)).abs();
        rd <= 1 && gd <= 1 && bd <= 1 && ad == 0
    }

    #[test]
    fn test_identities() {
        for r in 0..16 {
            for g in 0..16 {
                for b in 0..16 {
                    let original = egui::Color32::from_rgb(r * 17, g * 17, b * 17);
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

    #[test]
    fn test_hue_shift_1() {
        let red: Hsla = egui::Color32::RED.into();
        let green: Hsla = egui::Color32::GREEN.into();
        let blue: Hsla = egui::Color32::BLUE.into();

        {
            let red_to_green_shift = Hsla::calc_hue_shift(egui::Color32::RED, egui::Color32::GREEN);
            let shifted = Hsla::new(
                Hsla::hue_shift(red.h, red_to_green_shift),
                red.s,
                red.l,
                red.a,
            );
            assert_eq!(green, shifted);
        }

        {
            let green_to_blue_shift =
                Hsla::calc_hue_shift(egui::Color32::GREEN, egui::Color32::BLUE);
            let shifted = Hsla::new(
                Hsla::hue_shift(green.h, green_to_blue_shift),
                green.s,
                green.l,
                green.a,
            );
            assert_eq!(blue, shifted);
        }

        {
            let blue_to_red_shift = Hsla::calc_hue_shift(egui::Color32::BLUE, egui::Color32::RED);
            let shifted = Hsla::new(
                Hsla::hue_shift(blue.h, blue_to_red_shift),
                blue.s,
                blue.l,
                blue.a,
            );
            assert_eq!(red, shifted);
        }
    }

    #[test]
    fn calc_gamma_tables_properly() {
        const ONE_USIZE: usize = ONE_U16 as usize;

        let table_p5_to_p25 = Hsla::calc_gamma_table(0.5, 0.25);
        assert_eq!(ONE_USIZE + 1, table_p5_to_p25.len());
        assert_eq!(0, table_p5_to_p25[0]);
        assert_eq!(ONE_U16, table_p5_to_p25[ONE_USIZE]);
        assert_eq!(ONE_U16 / 4, table_p5_to_p25[ONE_USIZE / 2]);
        let mut last: u16 = 0;
        // All values should be increasing.
        for value in table_p5_to_p25 {
            assert!(value >= last);
            last = value;
        }

        let table_s0 = Hsla::calc_gamma_table(0.0, 0.5);
        let table_s1 = Hsla::calc_gamma_table(1.0, 0.5);
        let table_t0 = Hsla::calc_gamma_table(0.0, 0.0);
        let table_t1 = Hsla::calc_gamma_table(0.0, 1.0);

        assert_eq!(ONE_USIZE + 1, table_s0.len());
        for value in 0..=ONE_U16 {
            assert_eq!(value, table_s0[value as usize]);
        }
        assert_eq!(table_s0, table_s1);
        assert_eq!(table_s0, table_t0);
        assert_eq!(table_s0, table_t1);
    }

    #[test]
    fn do_hsla_transforms_properly() {
        let green = egui::Color32::from_rgb(0, 255, 0);
        let dblue = egui::Color32::from_rgb(0, 0, 128);
        let green_to_dblue = Hsla::calc_hsla_transform(green, dblue);

        let identity_result = green_to_dblue(&green);
        let identity_result_hsla: Hsla = identity_result.into();
        assert!(
            is_close(&identity_result, &dblue),
            "expected = {:?} actual = {:?} hsla = {:?}",
            dblue,
            identity_result,
            identity_result_hsla
        );

        let new_candidate = egui::Color32::from_rgb(32, 128, 64);
        let new_result = green_to_dblue(&new_candidate);
        let new_result_hsla: Hsla = new_result.into();
        // In,  { h: 2390, s: 615, l: 321, a: 255 }
        // Out, { h: 4438, s: 615, l: 101, a: 255 }
        // Orig lum = 31%,  New lum = 9.8%,  gamma =~ 2,  .31^2 = 9.6%.  9.8% =~ 9.6
        // It's what's expected - not 100% sure it's what I want wanted.
        let expected = egui::Color32::from_rgb(20, 10, 40);

        assert!(
            is_close(&expected, &new_result),
            "expected = {:?} actual = {:?} hsla_in = {:?} hsla_out = {:?}",
            expected,
            new_result,
            new_candidate,
            new_result_hsla
        );
    }

    #[test]
    fn calculate_hue_shift_properly() {
        use egui::Color32;
        // Original artwork is mostly green, but we want to make it mostly blue
        let original_green = Color32::from_rgb(0, 255, 0);
        let target_blue = Color32::from_rgb(0, 0, 255);

        // Convert the shift needed to do that in HSLA space
        let shift = Hsla::calc_hue_shift(original_green, target_blue);

        // Test: Convert a red tinged green with white saturation
        let input_green = egui::Color32::from_rgb(152, 255, 127);
        let hsla_green: Hsla = input_green.into();

        // Do the shift in HSLA space
        let green_shifted = Hsla {
            h: (hsla_green.h + shift) % (1024 * 6),
            s: hsla_green.s,
            l: hsla_green.l,
            a: hsla_green.a,
        };

        // Expected - a green tinged blue with white saturation
        let expected_blue = Color32::from_rgb(126, 151, 255);
        assert_eq!(expected_blue, green_shifted.into());
    }
}

pub struct Hsla {
    pub h: u16,
    pub s: u8,
    pub l: u8,
    pub a: u8,
}

impl From<&egui::Color32> for Hsla {
    fn from(item: &egui::Color32) -> Self {
        let r: i32 = i32::from(item.r());
        let g: i32 = i32::from(item.g());
        let b: i32 = i32::from(item.b());

        let min: i32 = core::cmp::min(core::cmp::min(r, g), b);
        let max: i32 = core::cmp::max(core::cmp::max(r, g), b);

        let l: i32 = (min + max) / 2;

        if min == max {
            return Hsla {
                h: 0,
                s: 0,
                l: u8::try_from(l).unwrap(),
                a: item.a(),
            };
        }

        let half: i32 = 128;
        let two: i32 = 512;
        let four: i32 = 1024;

        let s2: i32 = if l <= half {
            ((max - min) << 8) / (max + min)
        } else {
            ((max - min) << 8) / (two - max - min)
        };

        let s = if s2 == 256 { 255 } else { s2 };

        let ht: i32 = if r == max {
            ((g - b) << 8) / (max - min)
        } else if g == max {
            two + ((b - r) << 8) / (max - min)
        } else {
            four + ((r - g) << 8) / (max - min)
        };

        let h = (ht + 256 * 6) % (256 * 6);

        std::assert!(h >= 0);
        std::assert!(h <= 256 * 6);

        Hsla {
            h: u16::try_from(h).unwrap(),
            s: u8::try_from(s).unwrap(),
            l: u8::try_from(l).unwrap(),
            a: item.a(),
        }
    }
}

impl From<Hsla> for egui::Color32 {
    // https://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/

    fn from(val: Hsla) -> Self {
        if val.s == 0 {
            return egui::Color32::from_rgba_premultiplied(val.l, val.l, val.l, val.a);
        }
        let half: i32 = 128;
        let one: i32 = 256;
        let h: i32 = i32::from(val.h);
        let s: i32 = i32::from(val.s);
        let l: i32 = i32::from(val.l);

        let temp1: i32 = if l < half {
            (l * (one + s)) >> 8
        } else {
            l + s - ((l * s) >> 8)
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
            // we sometimes get small negatives.  skill issue/ bug.
            let tmp = hue_to_rgb_2(t1, t2, h).clamp(0, 255);
            u8::try_from(tmp).unwrap()
        }

        let r = hue_to_rgb(temp1, temp2, h + 512);
        let g = hue_to_rgb(temp1, temp2, h);
        let b = hue_to_rgb(temp1, temp2, h - 512);

        egui::Color32::from_rgba_premultiplied(r, g, b, val.a)
    }
}

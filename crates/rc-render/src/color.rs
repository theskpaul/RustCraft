//! Color-space helpers. Author colors in sRGB (what a color picker shows),
//! convert to linear before giving them to the GPU.

/// Convert one sRGB-encoded channel in [0, 1] to linear light.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert an sRGB RGB triple to linear. Alpha is never encoded, so it is not handled here.
pub fn srgb3_to_linear(rgb: [f32; 3]) -> [f32; 3] {
    [
        srgb_to_linear(rgb[0]),
        srgb_to_linear(rgb[1]),
        srgb_to_linear(rgb[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn endpoints_are_fixed() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!(close(srgb_to_linear(1.0), 1.0));
    }

    #[test]
    fn mid_grey_is_darker_in_linear() {
        // sRGB 0.5 is about 21% linear light.
        assert!(close(srgb_to_linear(0.5), 0.2140));
    }

    #[test]
    fn curve_is_continuous_at_the_knee() {
        let knee = 0.040_45_f32;
        let linear_part = knee / 12.92;
        let power_part = ((knee + 0.055) / 1.055).powf(2.4);
        assert!(close(linear_part, power_part));
    }

    #[test]
    fn curve_is_monotonic() {
        let mut prev = 0.0;
        for i in 1..=100 {
            let v = srgb_to_linear(i as f32 / 100.0);
            assert!(v > prev);
            prev = v;
        }
    }
}

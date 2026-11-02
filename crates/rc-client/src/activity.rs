//! Cheap per-frame input activity counters, used to correlate frame-time
//! spikes with what the user was doing. Plain integers only: no I/O here.

use crate::fly::FlyInput;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activity {
    /// Raw mouse-motion events received.
    pub mouse_events: u32,
    /// Key press/release events received (auto-repeat excluded).
    pub key_events: u32,
}

impl Activity {
    /// Return the current counts and reset to zero.
    pub fn take(&mut self) -> Activity {
        std::mem::take(self)
    }

    pub fn add(&mut self, other: Activity) {
        self.mouse_events += other.mouse_events;
        self.key_events += other.key_events;
    }
}

const KEY_NAMES: [&str; 7] = ["W", "S", "A", "D", "Space", "Ctrl", "Shift"];

/// Held fly keys as a bitmask; bit order matches KEY_NAMES.
pub fn held_mask(f: &FlyInput) -> u8 {
    [f.forward, f.back, f.left, f.right, f.up, f.down, f.boost]
        .iter()
        .enumerate()
        .fold(0u8, |m, (i, &down)| m | ((down as u8) << i))
}

/// Mask to text, e.g. "W+Shift", or "-" if nothing is held. Allocates.
pub fn mask_label(mask: u8) -> String {
    let held: Vec<&str> = KEY_NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, n)| *n)
        .collect();
    if held.is_empty() {
        "-".to_string()
    } else {
        held.join("+")
    }
}

pub fn keys_label(f: &FlyInput) -> String {
    mask_label(held_mask(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_returns_and_resets() {
        let mut a = Activity {
            mouse_events: 3,
            key_events: 2,
        };
        let got = a.take();
        assert_eq!(
            got,
            Activity {
                mouse_events: 3,
                key_events: 2
            }
        );
        assert_eq!(a, Activity::default());
    }

    #[test]
    fn add_sums_fields() {
        let mut a = Activity {
            mouse_events: 1,
            key_events: 1,
        };
        a.add(Activity {
            mouse_events: 4,
            key_events: 2,
        });
        assert_eq!(
            a,
            Activity {
                mouse_events: 5,
                key_events: 3
            }
        );
    }

    #[test]
    fn keys_label_idle_and_combo() {
        assert_eq!(keys_label(&FlyInput::default()), "-");
        let f = FlyInput {
            forward: true,
            boost: true,
            ..Default::default()
        };
        assert_eq!(keys_label(&f), "W+Shift");
    }

    #[test]
    fn mask_bits_and_label() {
        let f = FlyInput {
            forward: true,
            boost: true,
            ..Default::default()
        };
        assert_eq!(held_mask(&f), 0b100_0001);
        assert_eq!(mask_label(0), "-");
        assert_eq!(mask_label(0b100_0001), "W+Shift");
    }
}

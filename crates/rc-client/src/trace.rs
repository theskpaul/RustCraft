//! Per-frame trace recorded in memory and written as CSV on exit.
//! Enabled with RC_TRACE=<path>; zero cost when disabled (an Option check).

use crate::activity::mask_label;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct FrameRecord {
    pub frame: u32,
    /// Milliseconds since the program started.
    pub t_ms: f32,
    /// Milliseconds since the previous frame started.
    pub dt_ms: f32,
    pub acquire_ms: f32,
    pub submit_ms: f32,
    pub present_ms: f32,
    pub mouse_events: u16,
    pub key_events: u16,
    /// Bitmask of held fly keys (see activity::held_mask).
    pub held: u8,
}

/// Fixed-capacity recorder. Storage is allocated once up front so that
/// recording never reallocates (a Vec growth copy mid-run would itself
/// cause a frame hitch). When full, new rows are counted and dropped.
pub struct Trace {
    rows: Vec<FrameRecord>,
    cap: usize,
    dropped: u64,
}

impl Trace {
    pub fn new(cap: usize) -> Self {
        Self {
            rows: Vec::with_capacity(cap),
            cap,
            dropped: 0,
        }
    }

    pub fn push(&mut self, r: FrameRecord) {
        if self.rows.len() < self.cap {
            self.rows.push(r);
        } else {
            self.dropped += 1;
        }
    }

    pub fn recorded(&self) -> usize {
        self.rows.len()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Generic over any writer so tests can write to memory instead of disk.
    pub fn write_csv<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(
            w,
            "frame,t_ms,dt_ms,acquire_ms,submit_ms,present_ms,mouse_events,key_events,held_mask,held"
        )?;
        for r in &self.rows {
            writeln!(
                w,
                "{},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{}",
                r.frame,
                r.t_ms,
                r.dt_ms,
                r.acquire_ms,
                r.submit_ms,
                r.present_ms,
                r.mouse_events,
                r.key_events,
                r.held,
                mask_label(r.held)
            )?;
        }
        Ok(())
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut w = BufWriter::new(File::create(path)?);
        self.write_csv(&mut w)?;
        w.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(frame: u32) -> FrameRecord {
        FrameRecord {
            frame,
            t_ms: frame as f32 * 16.7,
            dt_ms: 16.7,
            acquire_ms: 16.0,
            submit_ms: 0.25,
            present_ms: 0.05,
            mouse_events: 2,
            key_events: 1,
            held: 0b100_0001, // W + Shift
        }
    }

    #[test]
    fn bounded_and_counts_dropped() {
        let mut t = Trace::new(2);
        for i in 0..5 {
            t.push(rec(i));
        }
        assert_eq!(t.recorded(), 2);
        assert_eq!(t.dropped(), 3);
    }

    #[test]
    fn csv_has_header_and_one_line_per_row() {
        let mut t = Trace::new(10);
        t.push(rec(0));
        t.push(rec(1));
        let mut out = Vec::new();
        t.write_csv(&mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("frame,t_ms"));
        assert!(lines[1].ends_with(",65,W+Shift"));
    }
}

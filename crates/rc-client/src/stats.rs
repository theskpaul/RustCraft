use std::time::Instant;

pub struct Report {
    pub frames: usize,
    pub avg_ms: f32,
    pub p50_ms: f32,
    pub p99_ms: f32,
    pub max_ms: f32,
}

impl Report {
    pub fn fps(&self) -> f32 {
        1000.0 / self.avg_ms
    }
    /// "1% low" FPS = FPS at the 99th-percentile frame time.
    pub fn low1_fps(&self) -> f32 {
        1000.0 / self.p99_ms
    }
}

pub fn percentile(sorted: &[f32], p: f32) -> f32 {
    let idx = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[idx]
}

pub fn summarize(dts_ms: &[f32]) -> Option<Report> {
    if dts_ms.is_empty() {
        return None;
    }
    let mut v = dts_ms.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(Report {
        frames: v.len(),
        avg_ms: v.iter().sum::<f32>() / v.len() as f32,
        p50_ms: percentile(&v, 0.50),
        p99_ms: percentile(&v, 0.99),
        max_ms: *v.last().unwrap(),
    })
}

pub struct FrameStats {
    last: Instant,
    window_start: Instant,
    window: Vec<f32>,
    all: Vec<f32>,
}

impl FrameStats {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last: now,
            window_start: now,
            window: Vec::new(),
            all: Vec::new(),
        }
    }

    /// Call once per presented frame. Returns a report about once per second.
    pub fn tick(&mut self) -> Option<Report> {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32() * 1000.0;
        self.last = now;
        self.window.push(dt);
        if self.all.len() < 1_000_000 {
            self.all.push(dt);
        }
        if (now - self.window_start).as_secs_f32() >= 1.0 {
            self.window_start = now;
            let r = summarize(&self.window);
            self.window.clear();
            r
        } else {
            None
        }
    }

    pub fn overall(&self) -> Option<Report> {
        summarize(&self.all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_basic() {
        let v: Vec<f32> = (1..=100).map(|x| x as f32).collect();
        let r = summarize(&v).unwrap();
        assert_eq!(r.frames, 100);
        assert_eq!(r.max_ms, 100.0);
        assert!((r.avg_ms - 50.5).abs() < 1e-4);
        assert_eq!(r.p99_ms, 99.0);
    }

    #[test]
    fn empty_is_none() {
        assert!(summarize(&[]).is_none());
    }
}

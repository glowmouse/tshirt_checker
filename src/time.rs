use web_time::SystemTime;

pub trait Time {
    fn reset(&mut self);
    fn ms_since_reset(&self) -> u32;
}

pub struct RealTime {
    time_at_reset: SystemTime,
}

impl Time for RealTime {
    fn reset(&mut self) {
        self.time_at_reset = SystemTime::now();
    }
    fn ms_since_reset(&self) -> u32 {
        self.time_at_reset
            .elapsed()
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap()
    }
}

impl Default for RealTime {
    fn default() -> Self {
        Self {
            time_at_reset: SystemTime::now(),
        }
    }
}

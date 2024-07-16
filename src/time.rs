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

#[derive(Default)]
pub struct FakeTime {
    time: u32,
}

impl Time for FakeTime {
    fn reset(&mut self) {
        self.time = 0;
    }
    fn ms_since_reset(&self) -> u32 {
        self.time
    }
}

impl FakeTime {
    #[cfg(test)]
    pub fn _advance(&mut self, time_to_advance: u32) {
        self.time += time_to_advance;
    }
}

use web_time::SystemTime;

pub trait Time {
    fn ms_since_start(&self) -> u64;
}

pub struct RealTime {
    time_at_reset: SystemTime,
}

impl Time for RealTime {
    fn ms_since_start(&self) -> u64 {
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
    time: u64,
}

impl Time for FakeTime {
    fn ms_since_start(&self) -> u64 {
        self.time
    }
}

impl FakeTime {
    #[cfg(test)]
    pub fn _advance(&mut self, time_to_advance: u64) {
        self.time += time_to_advance;
    }
}

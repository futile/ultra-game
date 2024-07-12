use std::time::Duration;

pub use bevy::prelude::*;

#[derive(Debug, Reflect)]
pub struct FiniteRepeatingTimer {
    timer: Timer,
    remaining_ticks: u32,
}

impl FiniteRepeatingTimer {
    pub fn new(tick_interval: Duration, num_ticks: u32) -> Self {
        Self {
            timer: Timer::new(tick_interval, TimerMode::Repeating),
            remaining_ticks: num_ticks,
        }
    }

    pub fn remaining_ticks(&self) -> u32 {
        self.remaining_ticks
    }

    pub fn is_finished(&self) -> bool {
        self.remaining_ticks() > 0
    }

    pub fn tick_get_fresh_ticks(&mut self, elapsed: Duration) -> u32 {
        if self.is_finished() {
            return 0;
        }

        self.timer.tick(elapsed);

        let fresh_ticks = self
            .timer
            .times_finished_this_tick()
            .min(self.remaining_ticks);

        // the `checked_sub()` is just an assertion, shouldn't hapen because we use `min()`
        // before
        self.remaining_ticks.checked_sub(fresh_ticks).unwrap();

        fresh_ticks
    }
}

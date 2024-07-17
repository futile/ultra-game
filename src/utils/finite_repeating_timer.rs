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
        self.remaining_ticks() == 0
    }

    pub fn remaining_time(&self) -> Duration {
        (self.remaining_ticks.saturating_sub(1)) * self.timer.duration() + self.timer.remaining()
    }

    #[must_use]
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
        self.remaining_ticks = self.remaining_ticks.checked_sub(fresh_ticks).unwrap();

        fresh_ticks
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::FiniteRepeatingTimer;

    #[test]
    fn zero_timer_works() {
        let mut timer = FiniteRepeatingTimer::new(Duration::ZERO, 0);

        assert!(timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::ZERO), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 0);
    }

    #[test]
    fn max_timer_works() {
        let mut timer = FiniteRepeatingTimer::new(Duration::MAX, 2);

        assert!(!timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 2);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::ZERO), 0);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 1);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 1);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 0);

        assert!(timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 0);
    }

    #[test]
    fn partial_ticking_works() {
        let mut timer = FiniteRepeatingTimer::new(Duration::from_secs(10), 2);

        assert!(!timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 2);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::ZERO), 0);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(5)), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(5)), 1);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(5)), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(20)), 1);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 0);

        assert!(timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 0);
    }

    #[test]
    fn multi_tick_works() {
        let mut timer = FiniteRepeatingTimer::new(Duration::from_secs(10), 3);

        assert!(!timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 3);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::ZERO), 0);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(20)), 2);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(5)), 0);
        assert_eq!(timer.tick_get_fresh_ticks(Duration::from_secs(20)), 1);

        assert_eq!(timer.tick_get_fresh_ticks(Duration::MAX), 0);

        assert!(timer.is_finished());
        assert_eq!(timer.remaining_ticks(), 0);
    }
}

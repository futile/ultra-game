pub mod egui_systems;

mod finite_repeating_timer;
use std::time::Duration;

pub use finite_repeating_timer::FiniteRepeatingTimer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitDuration {
    pub days: u32,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub tenths: u8,
    pub hundredths: u8,
    pub millis: u8,
}

impl SplitDuration {
    pub fn from_duration(duration: &Duration) -> SplitDuration {
        let as_millis = duration.as_millis();

        let millis: u8 = (as_millis % 10) as u8;
        let hundredths: u8 = (as_millis % 100 / 10) as u8;
        let tenths: u8 = (as_millis % 1000 / 100) as u8;
        let seconds: u8 = (as_millis % (60 * 1000) / 1000) as u8;
        let minutes: u8 = (as_millis % (60 * 60 * 1000) / (60 * 1000)) as u8;
        let hours: u8 = (as_millis % (24 * 60 * 60 * 1000) / (60 * 60 * 1000)) as u8;
        let days: u32 = (as_millis / (24 * 60 * 60 * 1000)) as u32;

        SplitDuration {
            days,
            hours,
            minutes,
            seconds,
            tenths,
            hundredths,
            millis,
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::SplitDuration;

    #[test]
    fn split_duration_works() {
        let days: u32 = 420;
        let hours: u8 = 13;
        let minutes: u8 = 37;
        let seconds: u8 = 33;
        let tenths: u8 = 5;
        let hundredths: u8 = 3;
        let millis: u8 = 1;

        let ddays = Duration::from_days(days as u64);
        let dhours = Duration::from_hours(hours as u64);
        let dminutes = Duration::from_mins(minutes as u64);
        let dseconds = Duration::from_secs(seconds as u64);
        let dtenths = Duration::from_millis(tenths as u64 * 100);
        let dhundredths = Duration::from_millis(hundredths as u64 * 10);
        let dmillis = Duration::from_millis(millis as u64);

        let dcombined = ddays + dhours + dminutes + dseconds + dtenths + dhundredths + dmillis;
        let split = SplitDuration::from_duration(&dcombined);

        let expected = SplitDuration {
            days,
            hours,
            minutes,
            seconds,
            tenths,
            hundredths,
            millis,
        };

        assert_eq!(split, expected);
    }
}

//! Pure count conversion and error diffusion. No platform dependencies.
use crate::profiles::{Mode, Profile};

pub fn exact_counts(profile: &Profile) -> Result<f64, String> {
    let count = match profile.mode {
        Mode::Game => 180.0 / (positive(profile.sensitivity)? * positive(profile.yaw())?),
        Mode::Distance => positive(profile.cm360)? / 2.0 / 2.54 * positive(profile.dpi)?,
        Mode::Raw => positive(profile.raw_counts)?,
    };
    if !count.is_finite() || !(1.0..=1_000_000.0).contains(&count) {
        return Err("Choose settings between 1 and 1,000,000 counts per turn.".into());
    }
    Ok(count)
}

fn positive(value: f64) -> Result<f64, String> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err("Enter a number greater than zero.".into())
    }
}

#[derive(Default)]
pub struct Remainder(f64);
impl Remainder {
    pub fn next(&mut self, exact: f64) -> i32 {
        let accumulated = exact + self.0;
        let rounded = accumulated.round();
        self.0 = accumulated - rounded;
        rounded as i32
    }
}

/// Integer error diffusion: every partition sums to exactly `total`, in either direction.
pub fn steps(total: i32, n: u32) -> Vec<i32> {
    let n = n.max(1) as i64;
    let mut previous = 0;
    (1..=n)
        .map(|i| {
            let cumulative = i * i64::from(total) / n;
            let delta = cumulative - previous;
            previous = cumulative;
            delta as i32
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversions() {
        let mut p = Profile {
            game: "custom".into(),
            sensitivity: 1.5,
            custom_yaw: 0.022,
            ..Profile::default()
        };
        assert_eq!(exact_counts(&p).unwrap().round(), 5455.0);
        p.mode = Mode::Distance;
        p.cm360 = 25.4;
        p.dpi = 800.0;
        assert!((exact_counts(&p).unwrap() - 4000.0).abs() < 1e-9);
        p.mode = Mode::Raw;
        p.raw_counts = 1234.5;
        assert_eq!(exact_counts(&p).unwrap(), 1234.5);
    }
    #[test]
    fn rejects_invalid_and_overflow() {
        for sensitivity in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e-100] {
            assert!(exact_counts(&Profile {
                sensitivity,
                ..Profile::default()
            })
            .is_err());
        }
    }
    #[test]
    fn rounding_does_not_drift() {
        let mut remainder = Remainder::default();
        let exact = 180.0 / (1.5 * 0.022);
        let total: i64 = (0..10000).map(|_| i64::from(remainder.next(exact))).sum();
        assert!((total as f64 - exact * 10000.0).abs() <= 0.5);
    }
    #[test]
    fn partitions_keep_every_count() {
        for total in [-1000000, -5455, -1, 0, 1, 5455, 1000000] {
            for n in [0, 1, 2, 12, 101] {
                let s = steps(total, n);
                assert_eq!(s.iter().sum::<i32>(), total);
                assert!(s
                    .iter()
                    .all(|v| v.abs() <= (total.abs() / n.max(1) as i32) + 1));
            }
        }
    }
}

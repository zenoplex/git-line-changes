/// Provides getter methods.
pub trait StateAccess {
    fn get_stat(&self) -> &Stat;

    fn get_insertion(&self) -> u32 {
        self.get_stat().insertion
    }

    fn get_deletion(&self) -> u32 {
        self.get_stat().deletion
    }

    fn get_change_delta(&self) -> i32 {
        self.get_stat().change_delta
    }
}

/// Immutable struct that represents the statistics of changed lines.
#[derive(Debug, Clone, Default)]
pub struct Stat {
    insertion: u32,
    deletion: u32,
    change_delta: i32,
}

impl Stat {
    pub fn new(insertion: u32, deletion: u32) -> Stat {
        Stat {
            insertion,
            deletion,
            change_delta: insertion as i32 - deletion as i32,
        }
    }
}

impl StateAccess for Stat {
    fn get_stat(&self) -> &Stat {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_insertion() {
        assert_eq!(Stat::new(10, 5).get_insertion(), 10);
    }

    #[test]
    fn test_get_deletion() {
        assert_eq!(Stat::new(10, 5).get_deletion(), 5);
    }

    #[test]
    fn test_get_change_delta() {
        let test_cases = [((10, 5), 5), ((0, 0), 0), ((5, 10), -5)];

        for &((insertion, deletion), expected_delta) in &test_cases {
            assert_eq!(
                Stat::new(insertion, deletion).get_change_delta(),
                expected_delta
            );
        }
    }
}

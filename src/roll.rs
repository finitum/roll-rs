use crate::filtermodifier::FilterModifier;
use rand_core::RngCore;
use std::num::NonZeroU64;

pub struct Roll {
    pub vals: Vec<u64>,
    pub total: i64,
    pub sides: NonZeroU64,
}

pub fn roll_die(
    times: u64,
    sides: NonZeroU64,
    fm: FilterModifier<u64>,
    mut rng: impl RngCore,
) -> Roll {
    let mut rolls = Vec::new();
    let range = (sides.get() - 1) + 1;
    for _ in 0..times {
        let roll = rng.next_u64() % range + 1;
        rolls.push(roll);
    }

    rolls.sort();

    match fm {
        FilterModifier::KeepLowest(i) => {
            rolls.truncate(i as usize);
        }
        FilterModifier::KeepHighest(i) => {
            rolls.reverse();
            rolls.truncate(i as usize);
        }
        FilterModifier::DropLowest(i) => {
            rolls.reverse();
            rolls.truncate(rolls.len() - i.min(rolls.len() as u64) as usize)
        }
        FilterModifier::DropHighest(i) => {
            rolls.truncate(rolls.len() - i.min(rolls.len() as u64) as usize)
        }
        FilterModifier::None => {}
    }

    Roll {
        total: rolls.iter().sum::<u64>() as i64,
        vals: rolls,
        sides,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::{Error, RngCore};

    struct DeterministicRng {
        value: i64,
    }

    impl DeterministicRng {
        pub fn new() -> Self {
            Self { value: -1 }
        }
    }

    impl RngCore for DeterministicRng {
        fn next_u32(&mut self) -> u32 {
            self.value += 1;
            self.value as u32
        }

        fn next_u64(&mut self) -> u64 {
            self.value += 1;
            self.value as u64
        }

        fn fill_bytes(&mut self, _: &mut [u8]) {
            unimplemented!()
        }

        fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), Error> {
            unimplemented!()
        }
    }

    #[test]
    fn test_kl() {
        let roll = roll_die(
            6,
            NonZeroU64::new(6).unwrap(),
            FilterModifier::KeepLowest(3),
            DeterministicRng::new(),
        );

        assert_eq!(roll.vals.len(), 3);
        assert_eq!(roll.total, 6);
    }

    #[test]
    fn test_dl_overflow() {
        let roll = roll_die(
            100,
            NonZeroU64::new(6).unwrap(),
            FilterModifier::DropLowest(300),
            DeterministicRng::new(),
        );
        assert_eq!(roll.vals.len(), 0);
        assert_eq!(roll.total, 0);
    }
}

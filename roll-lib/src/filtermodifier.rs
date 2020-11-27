use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum FilterModifier<T> {
    KeepLowest(T),
    KeepHighest(T),
    DropLowest(T),
    DropHighest(T),
    None,
}

impl<T: Display> Display for FilterModifier<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeepLowest(v) => {
                write!(f, "kl")?;
                v.fmt(f)?
            }
            Self::KeepHighest(v) => {
                write!(f, "kl")?;
                v.fmt(f)?
            }
            Self::DropLowest(v) => {
                write!(f, "dl")?;
                v.fmt(f)?
            }
            Self::DropHighest(v) => {
                write!(f, "dh")?;
                v.fmt(f)?
            }
            Self::None => {}
        }

        Ok(())
    }
}

impl<T> FilterModifier<T> {
    pub(crate) fn map<F, U>(self, f: F) -> FilterModifier<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::KeepLowest(i) => FilterModifier::KeepLowest(f(i)),
            Self::KeepHighest(i) => FilterModifier::KeepHighest(f(i)),
            Self::DropHighest(i) => FilterModifier::DropHighest(f(i)),
            Self::DropLowest(i) => FilterModifier::DropLowest(f(i)),
            Self::None => FilterModifier::None,
        }
    }
}

impl<T, E> FilterModifier<Result<T, E>> {
    pub(crate) fn swap(self) -> Result<FilterModifier<T>, E> {
        Ok(match self {
            FilterModifier::KeepLowest(i) => FilterModifier::KeepLowest(i?),
            FilterModifier::KeepHighest(i) => FilterModifier::KeepHighest(i?),
            FilterModifier::DropLowest(i) => FilterModifier::DropLowest(i?),
            FilterModifier::DropHighest(i) => FilterModifier::DropHighest(i?),
            FilterModifier::None => FilterModifier::None,
        })
    }
}

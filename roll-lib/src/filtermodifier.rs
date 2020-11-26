#[derive(Debug, PartialEq, Clone)]
pub enum FilterModifier<T> {
    KeepLowest(T),
    KeepHighest(T),
    DropLowest(T),
    DropHighest(T),
    None,
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

pub fn try_filter<I, P>(
    iter: I,
    mut predicate: P,
) -> impl Iterator<Item = Result<I::Item, Box<dyn std::error::Error>>>
where
    I: IntoIterator,
    P: FnMut(&I::Item) -> Result<bool, Box<dyn std::error::Error>>,
{
    iter.into_iter().filter_map(move |i| match predicate(&i) {
        Ok(true) => Some(Ok(i)),
        Err(error) => Some(Err(error)),
        _ => None,
    })
}

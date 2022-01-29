pub trait FindAny {
    /// Finds the first occurence of any of the given substrings in the string
    /// and returns its index and a reference to the found substring in the string itself.
    /// If none of the substrings are found, returns `None`.
    fn find_any<'a>(&'a self, substrings: &[impl AsRef<Self>]) -> Option<(usize, &'a Self)>;
}

impl FindAny for str {
    fn find_any<'a>(&'a self, substrings: &[impl AsRef<Self>]) -> Option<(usize, &'a Self)> {
        substrings
            .iter()
            .filter_map(|sub| self.match_indices(sub.as_ref()).next())
            .min_by_key(|&(i, _)| i)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_any() {
        assert_eq!("foobarbaz".find_any(&["bar", "baz"]), Some((3, "bar")));
        assert_eq!("foobazbar".find_any(&["bar", "baz"]), Some((3, "baz")));
        assert_eq!("foobarbaz".find_any(&["quox"]), None);
    }
}

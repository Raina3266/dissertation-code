use std::collections::HashSet;

/// Take a slice of sets, and remove any words which are present in every set
pub fn dedup_sets<'a>(sets: impl IntoIterator<Item = &'a mut HashSet<String>>) {
    let mut sets: Vec<_> = sets.into_iter().collect();
    let first = &sets[0];

    let mut words_in_all = HashSet::with_capacity(first.len());

    for word in first.iter() {
        if sets.iter().all(|set| set.contains(word)) {
            words_in_all.insert(word.clone());
        }
    }

    for set in sets.iter_mut() {
        for word in words_in_all.iter() {
            set.remove(word);
        }
    }
}

#[test]
fn dedup_test() {
    // hello is present in all, so should be the only word removed
    let set1 = HashSet::from_iter(["hello".into(), "world".into(), "foo".into()]);
    let set2 = HashSet::from_iter(["hello".into(), "world".into()]);
    let set3 = HashSet::from_iter(["hello".into(), "foo".into()]);

    let mut sets = [set1, set2, set3];

    dedup_sets(&mut sets);

    let [set1, set2, set3] = sets;

    // hello removed from all
    assert_eq!(set1, HashSet::from_iter(["world".into(), "foo".into()]));
    assert_eq!(set2, HashSet::from_iter(["world".into()]));
    assert_eq!(set3, HashSet::from_iter(["foo".into()]));
}

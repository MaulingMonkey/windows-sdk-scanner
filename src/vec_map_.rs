use std::borrow::*;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry::*;
use std::marker::PhantomData;
use std::mem::replace;



#[derive(Clone)]
pub struct VecMap<K, V> {
    keys:   BTreeMap<K, usize>,
    values: Vec<V>,
}

impl<K, V> VecMap<K, V> {
    pub fn keys                 (&    self) -> impl Iterator<Item = &    K> { self.keys.keys() }
    pub fn values_by_key        (&    self) -> impl Iterator<Item = &    V> { self.iter_by_key().map(|(_k, v)| v) }
    //pub fn values_by_key_mut    (&mut self) -> impl Iterator<Item = &mut V> { self.iter_mut().map(|(_k, v)| v) }
    pub fn values_by_insert     (&    self) -> impl Iterator<Item = &    V> { self.values.iter() }
    pub fn values_by_insert_mut (&mut self) -> impl Iterator<Item = &mut V> { self.values.iter_mut() }

    pub fn iter_by_key<'s>(&'s self) -> impl Iterator<Item = (&'s K, &'s V)> + 's {
        let values = &self.values;
        self.keys.iter().map(move |(k, &idx)| (k, &values[idx]))
    }

    //pub fn iter_mut<'s>(&'s mut self) -> impl Iterator<Item = (&'s K, &'s mut V)> + 's {
    //    let values = &mut self.values;
    //    self.keys.iter().map(move |(k, idx)| (k, &mut values[*idx]))
    //}

    // iter_by_insert is ~impossible - but values_by_insert should be sufficient

    pub fn entry(&mut self, key: K) -> vec_map::Entry<'_, K, V> where K: Ord {
        match self.keys.entry(key) {
            Vacant(entry) => vec_map::Entry::Vacant(vec_map::VacantEntry {
                entry,
                values: &mut self.values,
                _marker: PhantomData
            }),
            Occupied(entry) => vec_map::Entry::Occupied(vec_map::OccupiedEntry {
                idx: *entry.get(),
                values: &mut self.values,
                entry,
                _marker: PhantomData
            }),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> where K: Ord {
        match self.keys.entry(key) {
            Occupied(entry) => Some(replace(self.values.get_mut(*entry.get())?, value)),
            Vacant(entry) => {
                entry.insert(self.values.len());
                self.values.push(value);
                None
            },
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let idx = *self.keys.get(key)?;
        self.values.get(idx)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let idx = *self.keys.get(key)?;
        self.values.get_mut(idx)
    }
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self { Self { keys: Default::default(), values: Default::default() } }
}



pub mod vec_map {
    use std::collections::btree_map;
    use std::marker::PhantomData;

    pub enum Entry<'a, K: 'a, V: 'a> {
        Vacant(VacantEntry<'a, K, V>),
        Occupied(OccupiedEntry<'a, K, V>),
    }

    impl<'a, K: 'a, V: 'a> Entry<'a, K, V> {
    }



    pub struct VacantEntry<'a, K: 'a, V: 'a> {
        pub(super) entry:   btree_map::VacantEntry<'a, K, usize>,
        pub(super) values:  &'a mut Vec<V>,
        pub(super) _marker: PhantomData<&'a mut (K, V)>,
    }

    impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
        pub fn insert(self, value: V) -> &'a mut V where K: Ord {
            self.entry.insert(self.values.len());
            self.values.push(value);
            self.values.last_mut().unwrap()
        }
    }



    pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
        #[allow(dead_code)] // XXX
        pub(super) entry:   btree_map::OccupiedEntry<'a, K, usize>,
        pub(super) values:  &'a mut Vec<V>,
        pub(super) idx:     usize,
        pub(super) _marker: PhantomData<&'a mut (K, V)>,
    }

    impl<'a, K: 'a, V: 'a> OccupiedEntry<'a, K, V> {
        pub fn get    (&    self) -> &    V { &    self.values[self.idx] }
        pub fn get_mut(&mut self) -> &mut V { &mut self.values[self.idx] }
    }
}

use std::iter::FromIterator;
use std::borrow::Borrow;
use std::hash::{ Hash, Hasher, DefaultHasher };
use std::mem;

const INITIAL_N_BUCKET_SIZE: usize = 1;

/* 
    We should put trait bounds only at the places where we implement the methods that use them
    rather then on the data structure itself.
*/
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}
/* In implementing this either we can have one bucket to save memory but look will be inefficient
    or multiple buckets as required (basically double ) for fast lookup and consume more memory then the earlier
*/
impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        // usually the bucket length is power of 2 , but here we will keep it empty and will be created on first insertion
        HashMap { buckets: Vec::new(), items: 0 }
    }
}

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    entry: &'a mut (K, V), // key does not need to store bcz the element already has the key
}
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    key: K, // need to keep key so it can use when pushes
    map: &'a mut HashMap<K, V>,
    bucket: usize,
}
impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> where K: Hash + Eq {
    pub fn insert(self, value: V) -> &'a mut V {
        if self.map.buckets.is_empty() || self.map.items > (4 * self.map.buckets.len()) / 5 {
            self.map.resize();
        }
        self.map.buckets[self.bucket].push((self.key, value));
        self.map.items += 1;
        &mut self.map.buckets[self.bucket].last_mut().unwrap().1
    }
}
pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V> where K: Hash + Eq {
    pub fn or_insert(self, value: V) -> &'a mut V {
        match self {
            Entry::Occupied(el) => &mut el.entry.1,
            Entry::Vacant(el) => el.insert(value),
        }
    }
    pub fn or_insert_with<F>(self, maker: F) -> &'a mut V where F: FnOnce() -> V {
        match self {
            Entry::Occupied(el) => &mut el.entry.1,
            Entry::Vacant(el) => el.insert(maker()),
        }
    }
    pub fn default(self) -> &'a mut V where V: Default {
        self.or_insert_with(Default::default)
    }
}

impl<K, V> HashMap<K, V> where K: Hash + Eq {
    fn get_bucket<Q>(&self, key: &Q) -> Option<usize> where K: Borrow<Q>, Q: Hash + Eq + ?Sized {
        if self.buckets.is_empty() {
            return None;
        }
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        // understand it again at 30:00
        Some((hasher.finish() % (self.buckets.len() as u64)) as usize) // index into the buckets
    }

    // Entry Api: We can get reference to where something will be inserted into the map
    pub fn entry<'a>(&'a mut self, key: K) -> Entry<'a, K, V> {
        // resize the map if the size is 80%
        if self.buckets.is_empty() || self.items > (4 * self.buckets.len()) / 5 {
            self.resize();
        }
        let bucket = self.get_bucket(&key).expect("empty bucket handled in get_bucket");

        /* 
        // cannot borrow `*bucket` as mutable more than once at a time
        match bucket.iter_mut().find(|&&mut (ref ekey, _)| ekey == &key) {
            Some(entry) => { Entry::Occupied(OccupiedEntry { entry }) }
            None => { Entry::Vacant(VacantEntry { key, bucket }) }
        }
        */
        // if let Some(entry) = bucket.iter_mut().find(|&&mut (ref ekey, _)| ekey == &key) {
        //     return Entry::Occupied(OccupiedEntry { entry: unsafe { &mut *(entry as *mut _) } }); // unsafe rust code
        // }

        match self.buckets[bucket].iter().position(|&(ref ekey, _)| ekey == &key) {
            Some(index) =>
                Entry::Occupied(OccupiedEntry {
                    entry: &mut self.buckets[bucket][index],
                }),
            None => Entry::Vacant(VacantEntry { map: self, key, bucket }),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // resize the map if the size is 80%
        if self.buckets.is_empty() || self.items > (4 * self.buckets.len()) / 5 {
            self.resize();
        }
        // hash the key
        let bucket = self.get_bucket(&key).expect("empty bucket handled in get_bucket");
        let bucket = &mut self.buckets[bucket]; // Rust let us overide variables

        // iterate through arary and find key that matches the key sent by the user
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }
        self.items += 1;
        bucket.push((key, value));
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V> where K: Borrow<Q>, Q: Eq + Hash + ?Sized {
        // ref to Q where K can be borrowed as Q(If has reference to one can ge t refere to otehr without conversion)
        // As Q has Hash an Eq that is same as K Hash and Eq. Q doesnot need to be sized
        // keep the buckets in sorted order to make searching fast
        let bucket = self.get_bucket(&key).expect("empty bucket handled in get_bucket");
        self.buckets[bucket]
            .iter()
            .find(|&(ekey, _)| { ekey.borrow() == key })
            .map(|&(_, ref evalue)| evalue)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized {
        let bucket = self.get_bucket(&key).expect("empty bucket handled in get_bucket");
        let bucket = &mut self.buckets[bucket];
        let ind = bucket.iter().position(|&(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(ind).1) // replaces removed el in vec with last el -> Fast but changes order of the vector
    }
    pub fn len(&self) -> usize {
        self.items
    }
    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool where K: Borrow<Q>, Q: Hash + Eq + ?Sized {
        self.get(key).is_some()
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_N_BUCKET_SIZE,
            n => 2 * n,
        };
        // setup new set of new buckets (Create new bucket)
        // let mut new_buckets = vec![Vec::new(), target_size]; does not work bcz K and V does not have Clone trait so Vec(K,V) also does not have Clone
        let mut new_buckets = Vec::with_capacity(target_size);
        // fill the vector now (Pass all the values to new buckets)
        new_buckets.extend((0..target_size).map(|_| Vec::new()));
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            // understand it again at 30:00
            let index = (hasher.finish() % (new_buckets.len() as u64)) as usize; // index into the buckets
            new_buckets[index].push((key, value));
        }
        // (Replace old buckets with new ones)
        let _ = mem::replace(&mut self.buckets, new_buckets);
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.bucket) {
                        Some(&(ref key, ref value)) => {
                            self.at += 1;
                            break Some((key, value));
                        }
                        None => {
                            self.bucket += 1;
                            self.at = 0;

                            continue;
                        }
                    }
                }
                None => {
                    break None;
                }
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter { map: self, bucket: 0, at: 0 }
    }
}
pub struct IntoIter<K, V> {
    map: HashMap<K, V>,
    bucket: usize,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get_mut(self.bucket) {
                Some(bucket) => {
                    match bucket.pop() {
                        Some(pair) => {
                            break Some(pair);
                        }
                        None => {
                            self.bucket += 1;
                            continue;
                        }
                    }
                }
                None => {
                    break None;
                }
            }
        }
    }
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { map: self, bucket: 0 }
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V> where K: Hash + Eq {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = HashMap::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

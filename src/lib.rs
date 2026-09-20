use std::borrow::Borrow;
use std::hash::{ Hash, Hasher, DefaultHasher };
use std::mem;

const INITIAL_N_BUCKET_SIZE: usize = 1;
pub struct Bucket<K, V> {
    //List key value pairs that hash to the bucket
    // Key has to be hashable and comparable
    items: Vec<(K, V)>,
}

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

impl<K, V> HashMap<K, V> where K: Hash + Eq {
    fn get_bucket<Q>(&self, key: &Q) -> usize where K: Borrow<Q>, Q: Hash + Eq + ?Sized {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        // understand it again at 30:00
        (hasher.finish() % (self.buckets.len() as u64)) as usize // index into the buckets
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // resize the map if the size is 80%
        if self.buckets.is_empty() || self.items > (4 * self.buckets.len()) / 5 {
            self.resize();
        }
        // hash the key
        let bucket = self.get_bucket(&key);
        let bucket = &mut self.buckets[bucket]; // Rust let us overide variables

        self.items += 1;
        // iterate through arary and find key that matches the key sent by the user
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }
        bucket.push((key, value));
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V> where K: Borrow<Q>, Q: Eq + Hash + ?Sized {
        // ref to Q where K can be borrowed as Q(If has reference to one can ge t refere to otehr without conversion)
        // As Q has Hash an Eq that is same as K Hash and Eq. Q doesnot need to be sized
        // keep the buckets in sorted order to make searching fast
        let bucket = self.get_bucket(key);
        self.buckets[bucket]
            .iter()
            .find(|&(ekey, _)| { ekey.borrow() == key })
            .map(|&(_, ref evalue)| evalue)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized {
        let bucket = self.get_bucket(key);
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
        mem::replace(&mut self.buckets, new_buckets);
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

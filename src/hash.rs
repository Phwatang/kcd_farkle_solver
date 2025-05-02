//! Traits and structs for objects that can be perfectly hashed

use std::marker::PhantomData;
use serde::{Deserialize, Serialize};

use crate::farkle::{DiceSide, FarkleScore, DiceSetSample};
use std::ops::{Index, IndexMut};

/// The associated Hash type from a struct that has implemented PerfectHashing trait
#[derive(Copy, Clone)]
pub struct PerfectHash<T> {
    hash: usize,
    associated: PhantomData<T>,
}
impl<T> PerfectHash<T> {
    pub fn new(hash: usize) -> Self {
        return Self {
            hash: hash,
            associated: PhantomData,
        }
    }
}

/// Implement conversion process PerfectHash->usize
impl<T> From<PerfectHash<T>> for usize {
    fn from(value: PerfectHash<T>) -> Self {
        return value.hash;
    }
}

/// Implementing perfect hashing capabilities on a set of objects a struct may represent.
pub trait PerfectHashing: Sized {
    /// The size of the set of the object the hashing will operate on.
    /// I.e the max hash value outputted is SET_SIZE-1
    const SET_SIZE: usize;

    fn to_perfhash(&self) -> PerfectHash<Self>;
    fn from_perfhash(hash: PerfectHash<Self>) -> Self;
}

/// A Hashmap where a perfect hashing function exists for the keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfectHashMap<K: PerfectHashing, V: Sized + Default> {
    map: Vec<V>,
    associated: PhantomData<K>,
}
impl<K: PerfectHashing, V: Sized + Default> PerfectHashMap<K, V> {
    pub fn new() -> Self {
        let mut vec = Vec::with_capacity(K::SET_SIZE);
        for _ in 0..K::SET_SIZE {
            vec.push(V::default());
        }
        return Self {
            map: vec,
            associated: PhantomData,
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (K, &V)> {
        return (0..K::SET_SIZE).into_iter()
            .zip(self.map.iter())
            .map(|(k, v)| (K::from_perfhash(PerfectHash::new(k)), v));
    }

    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (K, &mut V)> {
        return (0..K::SET_SIZE).into_iter()
            .zip(self.map.iter_mut())
            .map(|(k, v)| (K::from_perfhash(PerfectHash::new(k)), v));
    }
}
impl<K: PerfectHashing, V: Sized + Default> Index<K> for PerfectHashMap<K, V> {
    type Output = V;
    
    fn index(&self, index: K) -> &Self::Output {
        return &self.map[usize::from(index.to_perfhash())]
    }
}
impl<K: PerfectHashing, V: Sized + Default> IndexMut<K> for PerfectHashMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        return &mut self.map[usize::from(index.to_perfhash())]
    }
}


impl PerfectHashing for DiceSetSample {
    const SET_SIZE: usize = 7usize.pow(6);

    fn to_perfhash(&self) -> PerfectHash<Self> {
        let mut val = 0;
        for (i, dice_outcome) in self.sample.iter().enumerate() {
            let n;
            if let &Some(side) = dice_outcome {
                n = (side as usize) + 1;
            } else {
                n = 0;
            }
            val += n * 7usize.pow(i as u32);
        }
        return PerfectHash::new(val);
    }
    fn from_perfhash(hash: PerfectHash<Self>) -> Self {
        let mut n: usize = hash.into();
        let mut output = [None; 6];
        for slot in output.iter_mut() {
            let i = n % 7;
            n = n / 7;
            match i {
                0 => *slot = None,
                _ => *slot = Some(DiceSide::from(i as u8)),
            }
        }
        return Self::new(output);
    }
}

impl PerfectHashing for [bool; 6] {
    const SET_SIZE: usize = 2usize.pow(6);

    fn to_perfhash(&self) -> PerfectHash<Self> {
        let num = self.iter()
            .enumerate()
            .fold(0, |acc, (i, bit)| {
                acc + (*bit as usize) * 2usize.pow(i as u32)
            });
        return PerfectHash::new(num);
    }
    fn from_perfhash(hash: PerfectHash<Self>) -> Self {
        let mut num: usize = hash.into();
        let mut mask: [bool; 6] = [false; 6];
        for i in 0..6 {
            mask[i] = (num % 2) != 0;
            num = num / 2
        }
        return mask;
    }
}

const HASH_DIV: usize = 50;
impl PerfectHashing for FarkleScore {
    const SET_SIZE: usize = 6000 / HASH_DIV;

    fn to_perfhash(&self) -> PerfectHash<Self> {
        return PerfectHash::new(self.value as usize / HASH_DIV);
    }
    fn from_perfhash(hash: PerfectHash<Self>) -> Self {
        return Self::new((usize::from(hash) * HASH_DIV) as u32);
    }
}

impl<T1: PerfectHashing, T2: PerfectHashing> PerfectHashing for (T1, T2) {
    const SET_SIZE: usize = T1::SET_SIZE * T2::SET_SIZE;

    fn to_perfhash(&self) -> PerfectHash<Self> {
        return PerfectHash::new(usize::from(self.0.to_perfhash()) * T2::SET_SIZE + usize::from(self.1.to_perfhash()));
    }
    fn from_perfhash(hash: PerfectHash<Self>) -> Self {
        let n: usize = hash.into();
        let t2 = T2::from_perfhash(PerfectHash::new(n % T2::SET_SIZE));
        let t1 = T1::from_perfhash(PerfectHash::new(n / T2::SET_SIZE));
        return (t1, t2);
    }
}
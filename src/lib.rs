// mod ptrs;
use std::rc::Rc;
use std::mem;
use std::fmt;
#[macro_use]
extern crate log;

trait Dict<T> {
    type K;

    fn empty() -> Self;
    // TODO: Collision handler fn.
    fn insert(&mut self, key: Self::K, val: T);
    fn lookup(&self, key: &Self::K) -> Option<&T>;
}

#[derive(Clone,Debug,PartialOrd,Ord,PartialEq,Eq)]
pub enum Trie<T> {
    Empty,
    // Key * T
    Lf (u64, T),
    // Depth * Key * left * right
    Br (usize, u64, Rc<Trie<T>>, Rc<Trie<T>>),
}

impl<T:Clone+fmt::Debug> Trie<T> {
    fn ins(&mut self, key: u64, val: T) {
        println!("#insert: {:?} <- {:?}={:?}", self, key, val);
        match &mut *self {
            me@&mut Trie::Empty => *me = Trie::Lf(key, val),
            &mut Trie::Lf(k, ref mut v) if k == key => *v = val,
            me@&mut Trie::Lf(_, _) => {
                if let Trie::Lf(k, v) = mem::replace(me, Trie::Empty) {
                    let pfx = Self::mask(key , d);
                    let oldleaf = Trie::Lf(k, v);
                    let leftp = Self::zerobit(key, d);
                    println!("leftp? {:?}@{:?} -> {:?}", key, d, leftp);
                    let mut new : Self = if leftp {
                        Trie::Br(d, pfx, Rc::new(oldleaf), Rc::new(Trie::Empty))
                    } else {
                        Trie::Br(d, pfx, Rc::new(Trie::Empty), Rc::new(oldleaf))
                    };
                    println!("#insert/split: -> {:?}", new);
                    new.ins(key, val);
                    println!("#insert/done: -> {:?}", new);
                    mem::replace(me, new);
                } else {
                    unreachable!()
                };
            },
            &mut Trie::Br(depth, _pfx, ref mut l, ref mut r) => {
                let leftp = Self::zerobit(key, depth);
                let child = if leftp {
                    Rc::make_mut(l)
                } else {
                    Rc::make_mut(r)
                };
                println!("insert/child: left? {:?}; into:{:?}", leftp, child);
                child.ins(key, val)
            }
        };
        println!("#inserted: {:?}", self);
    }

    fn zerobit(key: u64, pos: usize) -> bool {
        key & (1<<pos) != 0
    }
    fn mask(key: u64, pos: usize) -> u64 {
        let mask = (1<<d)-1;
        key & mask
    }
    fn branch_bit(a: u64, b: u64) -> usize {
        let diff = a ^ b;
        
    }

    fn join(self, p0:u64, other: Self, p1:u64) -> Self {
        let m = Self::branch_bit(p0, p1);
        if Self::zerobit(p0, m) {
            Trie::Br(m, Self::mask(p0, m), Rc::new(self), Rc::new(other))
        } else {
            Trie::Br(m, Self::mask(p0, m), Rc::new(other), Rc::new(self))
        }
    }
}

impl<T:Clone+fmt::Debug> Dict<T> for Trie<T> {
    type K = u64;
    fn empty() -> Self {
        Trie::Empty
    }
    fn insert(&mut self, key: Self::K, val: T) {
        self.ins(key, val)
    }
    fn lookup(&self, key: &Self::K) -> Option<&T> {
        println!("#lookup: {:?} <- {:?}", self, key);
        match self {
            &Trie::Empty => None,
            &Trie::Lf(k, ref v) if k == *key => Some(v),
            &Trie::Lf(k, _) => None,
            &Trie::Br(_pfx, ref l, ref r) => 
                if Self::zerobit(*key, depth) {
                    l.lookup(key)
                } else {
                    r.lookup(key)
                },
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate quickcheck;
    extern crate env_logger;
    use std::collections::BTreeMap;
    use super::{Trie,Dict};

    #[test]
    fn it_works() {
        env_logger::init().unwrap_or(());
        fn prop_works(insert: Vec<u64>, probe: u64) -> () {
            let mut d = Trie::empty();
            let mut m = BTreeMap::new();
            for k in insert {
                d.insert(k, k);
                m.insert(k, k);
            }
            println!("m: {:?}; d: {:?}", m, d);
            let mres = m.get(&probe);
            let res = d.lookup(&probe);
            println!("eq? {:?}", res == mres);
            assert_eq!(res, mres);
        }
        quickcheck::quickcheck(prop_works as fn(Vec<u64>, u64) -> ());
    }
}

// mod ptrs;
use std::fmt;
use std::mem;
#[macro_use]
extern crate log;

pub trait Dict<T> {
    type K;

    fn empty() -> Self;
    fn insert(&mut self, key: Self::K, val: T);
    fn remove(&mut self, key: &Self::K) -> Option<T>;
    fn lookup(&self, key: &Self::K) -> Option<&T>;
}

#[derive(Clone,Debug,PartialOrd,Ord,PartialEq,Eq)]
pub enum Trie<T> {
    Empty,
    // Key * T
    Lf(u64, T),
    // Prefix * Mask * left * right
    Br(u64, u64, Box<Trie<T>>, Box<Trie<T>>),
}

// The trace! invocations end up with a grammar that matches: '^(e|B*[Llb])$'
// start := empty | tree
// empty := Empty
// tree := Matching branch
//
impl<T: fmt::Debug> Trie<T> {
    fn ins(&mut self, key: u64, val: T) {
        // debug!("#insert: {:?} <- {:?}={:?}", self, key, val);
        match self {
            &mut Trie::Empty => {
                trace!("e");
                *self = Trie::Lf(key, val);
            }
            &mut Trie::Lf(k, ref mut v) if k == key => {
                trace!("L");
                *v = val;
            }
            &mut Trie::Lf(_, _) => {
                trace!("l");
                self.join(Trie::Lf(key, val));
            }
            &mut Trie::Br(p, m, ref mut l, ref mut r) if Self::match_prefix(key, p, m) => {
                trace!("B");
                let leftp = Self::zerobit(key, m);
                // debug!("zerobit({:#b}, {:#b}) => {:?}; branch:{:?}", key, m, leftp, if leftp { &*l } else { &*r });
                if leftp {
                    l.ins(key, val);
                } else {
                    r.ins(key, val);
                };
            }
            &mut Trie::Br(_, _, _, _) => {
                trace!("b");
                self.join(Trie::Lf(key, val));
            }
        };
        // debug!("#inserted: {:?}", new);
    }

    fn zerobit(key: u64, msk: u64) -> bool {
        key & msk == 0
    }
    fn mask(key: u64, msk: u64) -> u64 {
        let mask = msk - 1;
        key & mask
    }
    fn branch_bit(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        let bb = diff & (!diff + 1);
        // debug!("branch_bit: a:{:#b}; b:{:#b}; diff:{:#b}; bb:{:#b}", a, b, diff, bb);
        assert_eq!(bb.count_ones(), 1);
        assert_eq!(Self::mask(a, bb), Self::mask(b, bb));

        bb
    }

    fn join(&mut self, t1: Self) {
        // debug!("join:{:#b}:{:?}; {:#b}:{:?}", p0, self, p1, t1);
        let t0 = mem::replace(self, Trie::Empty);
        let p0 = t0.prefix();
        let p1 = t1.prefix();
        let m = Self::branch_bit(p0, p1);
        // debug!("join branch mask:{:?}; samep: {:?}", m, Self::zerobit(p0, m));
        if Self::zerobit(p0, m) {
            *self = Self::br(Self::mask(p0, m), m, Box::new(t0), Box::new(t1))
        } else {
            *self = Self::br(Self::mask(p0, m), m, Box::new(t1), Box::new(t0))
        };

        // debug!("join: => {:?}", self );
    }

    fn prefix(&self) -> u64 {
        match self {
            &Trie::Empty => 0,
            &Trie::Lf(k, _) => k,
            &Trie::Br(p, _, _, _) => p,
        }
    }

    fn match_prefix(k: u64, p: u64, m: u64) -> bool {
        Self::mask(k, m) == p
    }
    fn br(prefix: u64, mask: u64, left: Box<Trie<T>>, right: Box<Trie<T>>) -> Self {
        match (&*left, &*right) {
            (&Trie::Empty, &Trie::Empty) => Trie::Empty,
            (&Trie::Empty, _) => *right,
            (_, &Trie::Empty) => *left,
            (_, _) => Trie::Br(prefix, mask, left, right),
        }
    }
}

impl<T: Clone + fmt::Debug> Trie<T> {
    fn del(&self, key: &u64) -> (Self, Option<T>) {
        // debug!("#delert: {:?} <- {:?}", self, key);
        let new = match &*self {
            &Trie::Empty => (Trie::Empty, None),
            &Trie::Lf(k, ref val) if &k == key => (Trie::Empty, Some(val.clone())),
            &Trie::Lf(_, _) => (self.clone(), None),
            &Trie::Br(p, m, ref l, ref r) if Self::match_prefix(*key, p, m) => {
                let leftp = Self::zerobit(*key, m);
                // debug!("zerobit({:#b}, {:#b}) => {:?}; branch:{:?}", key, m, leftp, if leftp { l } else { r });
                if leftp {
                    let (left, removed) = l.del(key);
                    let new = Self::br(p, m, Box::new(left), r.clone());
                    (new, removed)
                } else {
                    let (right, removed) = r.del(key);
                    let new = Self::br(p, m, l.clone(), Box::new(right));
                    (new, removed)
                }
            }
            &Trie::Br(_, _, _, _) => (self.clone(), None),
        };
        // debug!("#delerted: {:?}", new);
        new
    }
}

impl<T: Clone + fmt::Debug> Dict<T> for Trie<T> {
    type K = u64;
    fn empty() -> Self {
        Trie::Empty
    }
    fn insert(&mut self, key: Self::K, val: T) {
        self.ins(key, val);
    }
    fn lookup(&self, key: &Self::K) -> Option<&T> {
        // debug!("#lookup: {:?} <- {:#b}", self, key);
        match self {
            &Trie::Empty => None,
            &Trie::Lf(k, ref v) if k == *key => Some(v),
            &Trie::Lf(_, _) => None,
            &Trie::Br(p, m, _, _) if !Self::match_prefix(*key, p, m) => None,
            &Trie::Br(_, m, ref l, ref r) => {
                let leftp = Self::zerobit(*key, m);
                let branch = if leftp {
                    l
                } else {
                    r
                };
                // debug!("zerobit({:#b}, {:#b}) => {:?}; branch:{:?}", key, m, leftp, branch);
                branch.lookup(key)
            }
        }
    }
    fn remove(&mut self, key: &Self::K) -> Option<T> {
        let (new, removed) = self.del(key);
        *self = new;
        removed
    }
}

#[cfg(test)]
mod tests {
    extern crate quickcheck;
    extern crate env_logger;
    use std::collections::BTreeMap;
    use super::{Trie, Dict};

    #[test]
    fn it_works() {
        env_logger::init().unwrap_or(());
        fn prop_works(insert: Vec<(u64, u64)>, probe: u64) -> () {
            let mut d = Trie::empty();
            let mut m = BTreeMap::new();
            for (k, v) in insert {
                println!("");
                d.insert(k, v);
                m.insert(k, v);
            }
            debug!("m: {:?}; d: {:?}", m, d);
            let mres = m.get(&probe);
            let res = d.lookup(&probe);
            debug!("eq? {:?}", res == mres);
            assert_eq!(res, mres);
        }
        quickcheck::quickcheck(prop_works as fn(Vec<(u64, u64)>, u64) -> ());
    }

    #[test]
    fn should_add_remove() {
        env_logger::init().unwrap_or(());
        fn prop_works(insert: Vec<(u64, u64)>, remove: Vec<u64>, probe: u64) -> () {
            debug!("{:?}", (&insert, &remove, &probe));
            let mut d = Trie::empty();
            let mut m = BTreeMap::new();
            for (k, v) in insert {
                d.insert(k, v);
                m.insert(k, v);
            }
            debug!("m: {:?}; d: {:?}", m, d);
            let mut ours = Vec::new();
            let mut theirs = Vec::new();
            for k in remove {
                ours.push(d.remove(&k));
                theirs.push(m.remove(&k));
            }

            let mres = m.get(&probe);
            let res = d.lookup(&probe);
            debug!("eq? {:?}", res == mres);
            debug!("removed {:?} == {:?} -> {:?}", ours, theirs, ours == theirs);
            debug!("");
            assert_eq!((res, ours), (mres, theirs));
        }
        quickcheck::quickcheck(prop_works as fn(Vec<(u64, u64)>, Vec<u64>, u64) -> ());
    }
}

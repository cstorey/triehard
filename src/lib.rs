// mod ptrs;
use std::rc::Rc;
use std::fmt;
#[macro_use]
extern crate log;

trait Dict<T> {
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
    Lf (u64, T),
    // Prefix * Mask * left * right
    Br (u64, u64, Rc<Trie<T>>, Rc<Trie<T>>),
}

impl<T:Clone+fmt::Debug> Trie<T> {
    fn ins(&self, key: u64, val: T) -> Self {
        debug!("#insert: {:?} <- {:?}={:?}", self, key, val);
        let new = match &*self {
            &Trie::Empty => Trie::Lf(key, val),
            &Trie::Lf(k, _) if k == key => Trie::Lf(key, val),
            &Trie::Lf(j, _) => {
                Self::join(key, Trie::Lf(key, val), j, self.clone())
            },
            &Trie::Br(p, m, ref l, ref r) if Self::match_prefix(key, p, m) => {
                let leftp = Self::zerobit(key, m);
                debug!("zerobit({:#b}, {:#b}) => {:?}; branch:{:?}", key, m, leftp, if leftp { l } else { r });
                if leftp {
                    Self::br(p, m, Rc::new(l.ins(key, val)), r.clone())
                } else {
                    Self::br(p, m, l.clone(), Rc::new(r.ins(key, val)))
                }
            },
            &Trie::Br(p, _, _, _) => {
                Self::join(key, Trie::Lf(key, val), p, self.clone())
            },
        };
        debug!("#inserted: {:?}", new);
        new
    }

    fn del(&self, key: &u64) -> (Self, Option<T>) {
        debug!("#delert: {:?} <- {:?}", self, key);
        let new = match &*self {
            &Trie::Empty => (Trie::Empty, None),
            &Trie::Lf(k, ref val) if &k == key => (Trie::Empty, Some(val.clone())),
            &Trie::Lf(j, _) => (self.clone(), None),
            &Trie::Br(p, m, ref l, ref r) if Self::match_prefix(*key, p, m) => {
                let leftp = Self::zerobit(*key, m);
                debug!("zerobit({:#b}, {:#b}) => {:?}; branch:{:?}", key, m, leftp, if leftp { l } else { r });
                if leftp {
                    let (left, removed) = l.del(key);
                    let new = Self::br(p, m, Rc::new(left), r.clone());
                    (new, removed)
                } else {
                    let (right, removed) = r.del(key);
                    let new = Self::br(p, m, l.clone(), Rc::new(right));
                    (new, removed)
                }
            },
            &Trie::Br(p, _, _, _) => {
                (self.clone(), None)
            },
        };
        debug!("#delerted: {:?}", new);
        new
    }

    fn zerobit(key: u64, msk: u64) -> bool {
        key & msk == 0
    }
    fn mask(key: u64, msk: u64) -> u64 {
        let mask = msk-1;
        key & mask
    }
    fn branch_bit(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        let bb = diff & (!diff+1);
        debug!("branch_bit: a:{:#b}; b:{:#b}; diff:{:#b}; bb:{:#b}",
            a, b, diff, bb);
        assert_eq!(bb.count_ones(), 1);
        assert_eq!(Self::mask(a, bb), Self::mask(b, bb));

        bb
    }

    fn join(p0:u64, t0:Self, p1:u64, t1:Self) -> Self {
        debug!("join:{:#b}:{:?}; {:#b}:{:?}", p0, t0, p1, t1);
        let m = Self::branch_bit(p0, p1);
        debug!("join branch mask:{:?}; samep: {:?}", m, Self::zerobit(p0, m));
        let ret = if Self::zerobit(p0, m) {
            Self::br(Self::mask(p0, m), m, Rc::new(t0), Rc::new(t1))
        } else {
            Self::br(Self::mask(p0, m), m, Rc::new(t1), Rc::new(t0))
        };

        debug!("join: => {:?}", ret );
        ret
    }

    fn match_prefix(k:u64, p:u64, m:u64) -> bool {
        Self::mask(k, m) == p
    }
    fn br(prefix: u64, mask: u64, left: Rc<Trie<T>>, right: Rc<Trie<T>>) -> Self {
        match (&*left, &*right) {
            (&Trie::Empty, &Trie::Empty) => Trie::Empty,
            (&Trie::Empty, _) => (*right).clone(),
            (_, &Trie::Empty) => (*left).clone(),
            (_, _) => Trie::Br(prefix, mask, left, right)
        }
    }
}

impl<T:Clone+fmt::Debug> Dict<T> for Trie<T> {
    type K = u64;
    fn empty() -> Self {
        Trie::Empty
    }
    fn insert(&mut self, key: Self::K, val: T) {
        let new = self.ins(key, val);
        *self = new;
    }
    fn lookup(&self, key: &Self::K) -> Option<&T> {
        debug!("#lookup: {:?} <- {:#b}", self, key);
        match self {
            &Trie::Empty => None,
            &Trie::Lf(k, ref v) if k == *key => Some(v),
            &Trie::Lf(_, _) => None,
            &Trie::Br(p, m, _, _) if !Self::match_prefix(*key, p, m) => None,
            &Trie::Br(_, m, ref l, ref r) => {
                let leftp = Self::zerobit(*key, m);
                let branch = if leftp { l } else { r };
                debug!("zerobit({:#b}, {:#b}) => {:?}; branch:{:?}", key, m, leftp, branch);
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
    use super::{Trie,Dict};

    #[test]
    fn it_works() {
        env_logger::init().unwrap_or(());
        fn prop_works(insert: Vec<(u64,u64)>, probe: u64) -> () {
            let mut d = Trie::empty();
            let mut m = BTreeMap::new();
            for (k,v) in insert {
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
        fn prop_works(insert: Vec<(u64,u64)>, remove: Vec<u64>, probe: u64) -> () {
            debug!("{:?}", (&insert, &remove, &probe));
            let mut d = Trie::empty();
            let mut m = BTreeMap::new();
            for (k,v) in insert {
                d.insert(k, v);
                m.insert(k, v);
            }
            debug!("m: {:?}; d: {:?}", m, d);
            let mut ours = Vec::new();
            let mut theirs = Vec::new();
            for k in remove {
                ours.push(d.remove(&k));
                theirs.push( m.remove(&k));
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

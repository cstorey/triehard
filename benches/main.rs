#![feature(test)]
extern crate triehard;
extern crate test;
extern crate rand;

use triehard::{Trie,Dict};
use test::Bencher;
use std::collections::BTreeMap;

#[bench] fn trie_insert_seq_00001  ( b: &mut Bencher )  { trie_insert_seq_n   ( 1, b    )  }
#[bench] fn trie_insert_seq_0010   ( b: &mut Bencher )  { trie_insert_seq_n   ( 10, b   )  }
#[bench] fn trie_insert_seq_0100   ( b: &mut Bencher )  { trie_insert_seq_n   ( 100, b  )  }
#[bench] fn trie_insert_seq_1000   ( b: &mut Bencher )  { trie_insert_seq_n   ( 1000, b )  }
#[bench] fn btree_insert_seq_0001  ( b: &mut Bencher )  { btree_insert_seq_n  ( 1, b    )  }
#[bench] fn btree_insert_seq_0010  ( b: &mut Bencher )  { btree_insert_seq_n  ( 10, b   )  }
#[bench] fn btree_insert_seq_0100  ( b: &mut Bencher )  { btree_insert_seq_n  ( 100, b  )  }
#[bench] fn btree_insert_seq_1000  ( b: &mut Bencher )  { btree_insert_seq_n  ( 1000, b )  }

#[bench] fn trie_insert_rand_0001  ( b: &mut Bencher )  { trie_insert_rand_n  ( 1, b    )  }
#[bench] fn trie_insert_rand_0010  ( b: &mut Bencher )  { trie_insert_rand_n  ( 10, b   )  }
#[bench] fn trie_insert_rand_0100  ( b: &mut Bencher )  { trie_insert_rand_n  ( 100, b  )  }
#[bench] fn trie_insert_rand_1000  ( b: &mut Bencher )  { trie_insert_rand_n  ( 1000, b )  }
#[bench] fn btree_insert_rand_0001 ( b: &mut Bencher )  { btree_insert_rand_n ( 1, b    )  }
#[bench] fn btree_insert_rand_0010 ( b: &mut Bencher )  { btree_insert_rand_n ( 10, b   )  }
#[bench] fn btree_insert_rand_0100 ( b: &mut Bencher )  { btree_insert_rand_n ( 100, b  )  }
#[bench] fn btree_insert_rand_1000 ( b: &mut Bencher )  { btree_insert_rand_n ( 1000, b )  }



fn trie_insert_seq_n(count: u64, b: &mut Bencher) {
    let mut t = Trie::empty();
    b.iter(|| for x in 0..count { t.insert(x, x) })
}

fn btree_insert_seq_n(count: u64, b: &mut Bencher) {
    let mut t = BTreeMap::new();
    b.iter(|| for x in 0..count { t.insert(x, x); })
}

fn trie_insert_rand_n(count: u64, b: &mut Bencher) {
    let mut t = Trie::empty();
    let mut input = Vec::new();
    for _ in 0..count {
        input.push(rand::random());
    }
    b.iter(|| for x in input.iter() { t.insert(*x, *x); })
}

fn btree_insert_rand_n(count: u64, b: &mut Bencher) {
    let mut t = BTreeMap::new();
    let mut input = Vec::new();
    for _ in 0..count {
        input.push(rand::random::<u64>());
    }
    b.iter(|| for x in input.iter() { t.insert(*x, *x); })
}

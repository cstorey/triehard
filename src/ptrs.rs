use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

trait Ptr<T> {
    type PT : Deref<Target=T>;
    fn build(val:T) -> Self::PT;
}

#[derive(Debug)]
struct ArcP;
#[derive(Debug)]
struct RcP;

impl<T> Ptr<T> for ArcP {
    type PT = Arc<T>;
    fn build(val:T) -> Self::PT {
        Arc::new(val)
    }
}

impl<T> Ptr<T> for RcP {
    type PT = Rc<T>;
    fn build(val:T) -> Self::PT {
        Rc::new(val)
    }
}

#[derive(Debug)]
struct Thing<P:Ptr<T>, T>{
    val: P::PT
}

impl<P:Ptr<T>, T> Thing<P, T> {
    fn new(val:T) ->Self {
        Thing { val: P::build(val) }
    }
}

#[test]
fn should_compile() {
    let a: Thing<ArcP, u64> = Thing::new(1);
    let b: Thing<RcP, u64> = Thing::new(2);
    println!("a:{:?}", a);
    println!("b:{:?}", b);
}

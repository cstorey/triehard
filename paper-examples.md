# Paper extracts
Naturally, used under fair use, from http://ittc.ku.edu/~andygill/papers/IntMap98.pdf

## §2 Binary Tries

datatype ’a Dict =
    Empty
  | Lf of ’a
  | Br of ’a Dict * ’a
fun lookup (k, Empty) = NONE
  | lookup (k, Lf x) = SOME x
  | lookup (k, Br (t0,t1)) = if even k
    then lookup (k div 2, t0)
    else lookup (k div 2, t1)

fun br (Empty, Empty) = Empty
  | br (t0, t1) = Br (t0, t1)

> A reasonable alternative to the above scheme is for each LF node to store its actual key, and for `lookup` to not discard bits of the key...

```sml
datatype ’a Dict =
    Empty
  | Lf of int * ’a
  | Br of int * ’a Dict * ’a

fun lookup (k, Empty) = NONE
  | lookup (k, Lf (j, x)) = if j=k then SOME x else NONE
  | lookup (k, Br (m, t0,t1)) = if zeroBit (k, m)
    then lookup (k, t0)
    else lookup (k, t1)

fun br (m, Empty, Empty) = Empty
  | br (m, Empty, t as Lf _) = t
  | br (m, t as Lf _, Empty) = t
  | br (m, t0, t1) = Br (m, t0, t1)

fun zeroBit (k, m) = (andb (k, m) = 0)
```

## §3 Patricia Trees

```sml
datatype ’a Dict =
    Empty
  | Lf of int * ’a
  | Br of int * int * ’a Dict * ’a

fun lookup (k, Empty) = NONE
  | lookup (k, Lf (j, x)) = if j=k then SOME x else NONE
  | lookup (k, Br (p, m, t0,t1)) =
    if not (matchPrefix (k, p, m) then NONE
    else if zeroBit (k, m)
      then lookup (k, t0)
      else lookup (k, t1)

fun mask (k, m) = andb(k, m-1)

fun br (m, Empty, t) = t
  | br (m, t, Empty) = t
  | br (m, t0, t1) = Br (m, t0, t1)

fun zeroBit (k, m) = (andb (k, m) = 0)
```

## §4 Insertions and Merges

```sml
fun join (p0, t0, p1, t1) =
  let val m = branchingBit (p0, p1)
  in if zeroBit (p0,m) then Br (mask (p0, m), m, t0, t1)
                       else Br (mask (p0, m), m, t1, t0)
  end

fun insert c (k, x, t) = 
  let fun ins Empty = Lf (k, x)
	| ins (t as Lf (j, y)) = 
	    if j=k then Lf (k, c (x, y))
	    else join (k, Lf (k, x), j, t)
	| ins (t as Br (p,m,t0,t1) =
	    if matchPrefix (k, p, m) then
	      if zeroBit (k,m) then Br (p,m,ins t0,t1)
	                       else Br (p,m,ins t1,t0)
	    else join (k, Lf (k,x),p,t)
    in ins t end

fun branchingBit (p0, p1) = lowestBit (xorb (p0, p1))
fun lowestBit x = if odd then 1 else 2 * lowestBit (x div 2)
```

So the mask (`m` field in the `Br` constructor) is essentially `1 << depth_in_tree`.  In rust we can use `u64#trailing_zeros` and `u64#leading_zeros` in the big-endian case.

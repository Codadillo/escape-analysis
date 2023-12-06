type UInt = ();

fn iszero(n: UInt) -> UInt {
    invent(n)
}

fn one() -> UInt {
    invent()
}

fn zero() -> UInt {
    invent()
}

fn times(a: UInt, b: UInt) -> UInt {
    if iszero(a) {
        zero()
    } else {
        if iszero(b) {
            zero()
        } else {
            times_inner(zero(), a, b)
        }
    }
}

fn times_inner(acc: UInt, a: UInt, b: UInt) -> UInt {
    if iszero(b) {
        acc
    } else {
        times_inner(add(acc, a), a, sub(b, one()))
    }
}

fn add(a: UInt, b: UInt) -> UInt {
    invent(a, b)
}

fn sub(a: UInt, b: UInt) -> UInt {
    invent(a, b)
}

fn factorial(n: UInt) -> UInt {
    if iszero(n) {
        one()
    } else {
        times(n, factorial(sub(n, one())))
    }
}

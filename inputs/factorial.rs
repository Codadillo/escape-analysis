fn iszero(n: ()) -> () {
    invent(n)
}

fn one() -> () {
    invent()
}

fn times(a: (), b: ()) -> () {
    invent(a, b)
}

fn sub(a: (), b: ()) -> () {
    invent(a, b)
}

fn factorial(n: ()) -> () {
    if iszero(n) {
        one()
    } else {
        times(n, factorial(sub(n, one())))
    }
}

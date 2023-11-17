fn other(b) {
    tuple(invent(), invent(), b)
}

fn new(a, b, c) {
    let a = tuple(a, b, c);
    let c = tuple(a, c);

    let d = tuple(c, c);
    let e = other(b);

    tuple(d, e)
}
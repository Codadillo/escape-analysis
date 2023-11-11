fn identity(a) {
    a
}

fn use_it(a, b) {
    let a_ = identity(a);
    let b_ = identity(b);
    tuple(a_, b_, b)
}

fn identity(a) {
    a
}

fn xor_chain(a) {
    let a = identity(a);
    let a = identity(a);
    let a = identity(a);
    let a = identity(a);
    let a = identity(a);
    a
}

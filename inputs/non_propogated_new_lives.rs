fn make(a: (), b: ()) -> ((), (), ()) {
    tuple(a, b, invent())
}

fn non_prop() -> () {
    let a = make(invent(), invent());

    invent(a)
}

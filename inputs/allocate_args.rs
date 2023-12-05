fn make_data(condition, a, b) {
    if condition {
        tuple(a, b, make_data(invent(condition), a, b))
    } else {
        tuple()
    }
}

fn allocate_args(a, b) {
    make_data(invent(), a, b)
}

fn my_tuple() {
    tuple(invent(), invent())
}

fn use_it() {
    allocate_args(my_tuple(), invent())
}

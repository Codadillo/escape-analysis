fn function(a, b, c) {
    let hi = if a {
        b
    } else {
        c
    };

    tuple(hi, b)
}
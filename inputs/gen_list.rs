fn generate(condition) {
    if condition {
        tuple()
    } else {
        tuple(tuple(), generate(condition))
    }
}

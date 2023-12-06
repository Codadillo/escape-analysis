fn layer_one() -> ((), ()) {
    tuple(invent(), invent())
} 

fn layer_two() -> ((), ((), ())) {
    tuple(invent(), layer_one())
}

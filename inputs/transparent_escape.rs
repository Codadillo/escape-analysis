fn has_trans_in_graph(a, b) {
    let c = a;
    let d = if a {
        b
    } else {
        c
    };
    tuple(d)
}

fn function(a, b) {
    let h = has_trans_in_graph(a, b);
    let c = invent();
    tuple(c, h)
}

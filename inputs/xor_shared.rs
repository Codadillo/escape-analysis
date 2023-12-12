fn function(cond: (), a: (), b: ()) -> ((), ()) {
    if cond {
        a
    } else {
        b
    }
}

fn useit() -> ((), ()) {
    function(invent(), invent(), invent())
}

fn exampple(arg: ()) -> ((), (), ()) {
    tuple(tuple(arg, invent()), invent())
}

fn exa(arg: ((), (), (), ())) -> ((), (), (), ()) {
    if invent() {
        arg
    } else {
        tuple(invent(), invent(), invent(), invent())
    }
}

fn hhh() -> () {
    if invent() {
        invent()
    } else {
        invent()
    }
}

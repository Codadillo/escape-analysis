type List = [() | ((), List)];

fn generate(condition: ()) -> List {
    if condition {
        List(tuple())
    } else {
        List(tuple(tuple(), generate(condition)))
    }
}

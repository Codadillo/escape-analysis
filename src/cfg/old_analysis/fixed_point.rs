pub fn fixed_point<T: Clone + PartialEq>(state: &mut T, mut f: impl FnMut(&mut T, &T)) {
    loop {
        let old = state.clone();

        f(state, &old);

        if old == *state {
            break;
        }
    }
}

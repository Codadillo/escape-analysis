Cfg(_1: N, _2: M, _3: K) => N, M - 1, K:
0: {
        goto -> if _1 { 1 } else { 2 }
}

1: {
        let _4 = function(_2, _3);
        let _5 = function(_4, _2, _3);
        drop(_4);
        drop(_5);
        drop(_2);

        let _6 = function(_3);
        goto -> 3
}

2: {
        drop(_2);


        let _7 = function(_3);
        goto -> 3
}

3: {
        let _8 = ϕ(_6, _7);
        let _9 = function(_8, _1);
        let _0 = _9;
        return _0
}
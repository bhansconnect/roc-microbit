app "line_follow"
    packages { pf: "../platform" }
    imports [ pf.IO ]
    provides [ main ] to pf

main : IO.Input -> IO.Output
main = \{state, lightLeft, lightRight} ->
    speed = 20
    T speedLeft speedRight next =
        when T lightLeft lightRight is
            T Dark Dark ->
                T -speed -speed 0
            T Bright Dark ->
                T 0 -speed 1
            T Dark Bright ->
                T -speed 0 2
            T Bright Bright ->
                when state is
                    0 ->
                        T -speed -speed 0
                    1 ->
                        T 0 -speed 1
                    2 ->
                        T -speed 0 2
                    _ ->
                        T 0 0 3
    ll =
        when lightLeft is
            Dark ->
                1
            Bright ->
                0
    lr =
        when lightRight is
            Dark ->
                16
            Bright ->
                0
    {
        delayMS: 200,
        state: next,
        display: IO.displayNum (ll + lr),
        speedLeft: Num.toI8 speedLeft,
        speedRight: Num.toI8 speedRight,
    }

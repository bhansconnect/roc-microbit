app "fib"
    packages { pf: "../platform" }
    imports [ pf.IO ]
    provides [ main ] to pf

main : IO.Input -> IO.Output
main = \{state} ->
    n = state
    data = fib n 0 1
    # Once the screen is full reset to 0.
    next = if Num.isEven (Num.shiftRightBy 24 data) then
            n + 1
        else
            0
    {
        state: next,
        display: IO.displayNum data,
        speedLeft: 0,
        speedRight: 0,
    }

# the clever implementation requires join points
fib : U64, U64, U64 -> U64
fib = \n, a, b ->
    if n == 0 then
        a
    else
        fib (n - 1) b (a + b)

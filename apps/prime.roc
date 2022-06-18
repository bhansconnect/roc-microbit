app "prime"
    packages { pf: "../platform" }
    imports [ pf.IO ]
    provides [ main ] to pf

main : IO.Input -> IO.Output
main = \{state} ->
    n = state
    data = prevPrime n
    # # Once the screen is full reset to 0.
    next = if Num.isEven (Num.shiftRightBy 24 data) then
            n + 1
        else
            0
    {
        delayMS: 50,
        state: next,
        display: IO.displayNum data,
        speedLeft: 0,
        speedRight: 0,
    }

# This returns the highest prime number less than n.
prevPrime = \n ->
    # All prime numbers are odd except two
    if n <= 1 then
        0
    else if n == 2 then
        1
    else
        n2 = if Num.isOdd n then
                n - 2
            else
                n - 1
        prevPrimeHelper n2


prevPrimeHelper = \i ->
    if i < 2 then
        2
    else if Num.isEven i then
        prevPrimeHelper (i - 2)
    else
        innerLoop = \base, j ->
            if j * j > base then
                j
            else
                when Num.rem base j is
                    Ok rem ->
                        if rem == 0 then
                            j
                        else
                            innerLoop base (j + 2)
                    Err _ -> Num.maxU64
        innerJ = innerLoop i 3
        if innerJ * innerJ > i then
            i
        else
            prevPrimeHelper (i - 2)

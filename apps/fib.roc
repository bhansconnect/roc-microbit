app "fib"
    packages { pf: "../platform" }
    imports [ pf.IO ]
    provides [ main ] to pf

main : U8 -> IO.Output
main = \n ->
    data = fib n 0 1
    {
        data: data,
        display: IO.displayNum data
    }

# the clever implementation requires join points
fib : U8, U64, U64 -> U64
fib = \n, a, b ->
    if n == 0 then
        a
    else
        fib (n - 1) b (a + b)

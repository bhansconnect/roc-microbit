app "fib"
    packages { pf: "../platform" }
    imports [ pf.IO.{ Output, Display, Row } ]
    provides [ main ] to pf

main : U8 -> Output
main = \n ->
    data = fib n 0 1
    bit0 = getBit data 0
    bit1 = getBit data 1
    bit2 = getBit data 2
    bit3 = getBit data 3
    bit4 = getBit data 4
    bit5 = getBit data 5
    bit6 = getBit data 6
    bit7 = getBit data 7
    bit8 = getBit data 8
    bit9 = getBit data 9
    bit10 = getBit data 10
    bit11 = getBit data 11
    bit12 = getBit data 12
    bit13 = getBit data 13
    bit14 = getBit data 14
    bit15 = getBit data 15
    bit16 = getBit data 16
    bit17 = getBit data 17
    bit18 = getBit data 18
    bit19 = getBit data 19
    bit20 = getBit data 20
    bit21 = getBit data 21
    bit22 = getBit data 22
    bit23 = getBit data 23
    bit24 = getBit data 24
    {
        data: data,
        display: Display
            (Row bit24 bit23 bit22 bit21 bit20)
            (Row bit19 bit18 bit17 bit16 bit15)
            (Row bit14 bit13 bit12 bit11 bit10)
            (Row bit9 bit8 bit7 bit6 bit5)
            (Row bit4 bit3 bit2 bit1 bit0),
    }

getBit : U64, U64 -> U8
getBit = \num, index ->
    # TODO: convert this to Num.toU8 once it is added.
    if Num.isEven (Num.shiftRightBy index num) then
        0
    else
        1

# the clever implementation requires join points
fib : U8, U64, U64 -> U64
fib = \n, a, b ->
    if n == 0 then
        a
    else
        fib (n - 1) b (a + b)

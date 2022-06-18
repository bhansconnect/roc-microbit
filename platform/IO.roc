interface IO
    exposes [ Input, Output, Display, Row, displayNum ]
    imports []

Row : [
        Row U8 U8 U8 U8 U8,
    ]

Display : [
        Display Row Row Row Row Row,
    ]

LightLevel : [
        Dark,
        Bright,
    ]

State : U64

Input : {
        state: State,
        lightLeft : LightLevel,
        lightRight : LightLevel,
    }

Output : {
        delayMS: U64,
        state: State,
        display : Display,
        speedLeft: I8,
        speedRight: I8,
    }

displayNum : U64 -> Display
displayNum = \num ->
    bit0 = getBit num 0
    bit1 = getBit num 1
    bit2 = getBit num 2
    bit3 = getBit num 3
    bit4 = getBit num 4
    bit5 = getBit num 5
    bit6 = getBit num 6
    bit7 = getBit num 7
    bit8 = getBit num 8
    bit9 = getBit num 9
    bit10 = getBit num 10
    bit11 = getBit num 11
    bit12 = getBit num 12
    bit13 = getBit num 13
    bit14 = getBit num 14
    bit15 = getBit num 15
    bit16 = getBit num 16
    bit17 = getBit num 17
    bit18 = getBit num 18
    bit19 = getBit num 19
    bit20 = getBit num 20
    bit21 = getBit num 21
    bit22 = getBit num 22
    bit23 = getBit num 23
    bit24 = getBit num 24
    Display
        (Row bit24 bit23 bit22 bit21 bit20)
        (Row bit19 bit18 bit17 bit16 bit15)
        (Row bit14 bit13 bit12 bit11 bit10)
        (Row bit9 bit8 bit7 bit6 bit5)
        (Row bit4 bit3 bit2 bit1 bit0)


getBit : U64, U64 -> U8
getBit = \num, index ->
    # TODO: convert this to Num.toU8 once it is added.
    if Num.isEven (Num.shiftRightBy index num) then
        0
    else
        1

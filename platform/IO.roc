interface IO
    exposes [ Output, Display, Row ]
    imports []

Row : [
        Row U8 U8 U8 U8 U8,
    ]

Display : [
        Display Row Row Row Row Row,
    ]

Output : {
        data: U64,
        display : Display,
    }

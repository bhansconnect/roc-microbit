platform "examples/add"
    requires {} { main : U8 -> U64 }
    exposes []
    packages {}
    imports []
    provides [ mainForHost ]

mainForHost : U8 -> U64
mainForHost = \a -> main a

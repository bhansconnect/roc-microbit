platform "microbit"
    requires {} { main : U64 -> IO.Output }
    exposes []
    packages {}
    imports [ IO ]
    provides [ mainForHost ]

mainForHost : U64 -> IO.Output
mainForHost = \a -> main a

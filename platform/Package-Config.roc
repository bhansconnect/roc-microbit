platform "microbit"
    requires {} { main : U64 -> IO.Output }
    exposes []
    packages {}
    imports [ IO ]
    provides [ mainForHost ]

mainForHost : IO.Input -> IO.Output
mainForHost = \a -> main a

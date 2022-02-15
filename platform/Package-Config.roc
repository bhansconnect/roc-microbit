platform "microbit"
    requires {} { main : U8 -> IO.Output }
    exposes []
    packages {}
    imports [ IO ]
    provides [ mainForHost ]

mainForHost : U8 -> IO.Output
mainForHost = \a -> main a

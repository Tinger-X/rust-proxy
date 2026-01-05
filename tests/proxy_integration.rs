mod common;

// NoAuth tests
mod noauth {
    mod http10;
    mod http11;
    mod http2;
    mod https;
    mod websocket;
}

// Auth tests
mod auth {
    mod http10;
    mod http11;
    mod http2;
    mod https;
    mod websocket;
}

// Std tests
mod std {
    mod config;
    mod http10;
    mod http11;
    mod http2;
    mod https;
    mod websocket;
}

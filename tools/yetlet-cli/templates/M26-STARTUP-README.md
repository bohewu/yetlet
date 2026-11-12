
## Application address

Visit http://127.0.0.1:7878/ or http://127.0.0.1:7878/hello/World after starting
the app. The generated `App::serve()` call serves plain HTTP at the fixed
loopback address `127.0.0.1:7878`. It ignores `YETLET_PORT`, adds no startup
logging or implicit settings, and returns a bind error if that address is
unavailable. The CLI's build/run/watch settings do not configure this listener.

For an explicit address, change `.serve()` to `.serve_on("127.0.0.1:8080")`,
or pass a `std::net::SocketAddr`. These methods do not enable HTTPS.
For actual port reporting, port `0` with `listener.local_addr()`, graceful
shutdown, direct TLS, custom socket options or multiple listeners, use the
app-owned `into_router()` and native Tokio/Axum listener path. Add the direct
Axum dependency and Tokio `net` feature when using that native path, retaining
any dependencies already selected for asset routing. TLS and shutdown require
explicit application-owned integration.

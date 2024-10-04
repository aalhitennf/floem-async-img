### async-img

Asynchronous image widget for [Floem (git)](https://github.com/lapce/floem) that fetches image from url on background task or thread.

Enable runtime support with one of the feature flags: `tokio`, `async-std`, `smol`, or `thread` to use without any async runtime.

Enable cache with feature flag `cache`. Cache stores the fetched image(s) in memory, or on disk if configured so, reducing network fetches.

#### Examples

Async image with specific runtime:
`cargo run --example async --features tokio|async-std|smol|thread`

Async image with cache and specific runtime:
`cargo run --example async --features cache,{tokio|async-std|smol|thread}`

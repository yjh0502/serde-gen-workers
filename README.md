## [codegen.jyu.workers.dev](https://rustgen.jyu.workers.dev)

Generate `rust` serde bindings from JSON data. WIP.

## TODO

 - Handle ndjson: `serde-gen` already supports it
 - Improve error handling, especially with invalid JSON input. It seems returning Err(_) crashes workers runtime :(
 - Improve build process. `include_dir!` fails to build incrementally, so `cargo clean` is required before publishing, which hurts iteration time.


## Notes

 - `workers-rs` runtime does not seems to support static assets from kv store.
   It requires `__STATIC_CONTENT_MANIFEST` as well as `__STATIC_CONTENT`,
    as `wrangler` mangles resource path with content hash.
   Unfortunately `__STATIC_CONTENT_MANIFEST` is not available on rust runtime, so I embedded all static assets into `wasm` binary.
 - Maximum bundle size of CF worker is 1MiB. The app compiles down to 1.2MB, half wasm code and half static assets (html, css and js).
    Fortunately it could be published to workers.dev as compressed size is under 1MB.

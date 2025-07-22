/Users/williamseemueller/.cargo/bin/cargo run --color=always --package ais --bin ais --profile dev
warning: function `start_ais_stream_with_callbacks` is never used
--> src/ais.rs:78:8
|
78 | pub fn start_ais_stream_with_callbacks() {
|        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
|
= note: `#[warn(dead_code)]` on by default

warning: `ais` (bin "ais") generated 1 warning
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
Running `target/debug/ais`

thread 'main' panicked at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/js-sys-0.3.77/src/lib.rs:6063:9:
cannot access imported statics on non-wasm targets
stack backtrace:
0: __rustc::rust_begin_unwind
at /rustc/6b00bc3880198600130e1cf62b8f8a93494488cc/library/std/src/panicking.rs:697:5
1: core::panicking::panic_fmt
at /rustc/6b00bc3880198600130e1cf62b8f8a93494488cc/library/core/src/panicking.rs:75:14
2: js_sys::global::get_global_object::SELF::init::__wbg_static_accessor_SELF_37c5d418e4bf5819
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/js-sys-0.3.77/src/lib.rs:6063:9
3: js_sys::global::get_global_object::SELF::init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/js-sys-0.3.77/src/lib.rs:6063:9
4: core::ops::function::FnOnce::call_once
at /Users/williamseemueller/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
5: once_cell::unsync::Lazy<T,F>::force::{{closure}}
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:775:28
6: once_cell::unsync::OnceCell<T>::get_or_init::{{closure}}
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:591:57
7: once_cell::unsync::OnceCell<T>::get_or_try_init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:629:23
8: once_cell::unsync::OnceCell<T>::get_or_init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:591:19
9: once_cell::unsync::Lazy<T,F>::force
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:774:13
10: <wasm_bindgen::__rt::LazyCell<T> as core::ops::deref::Deref>::deref
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasm-bindgen-0.2.100/src/rt/mod.rs:56:9
11: wasm_bindgen::JsThreadLocal<T>::with
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasm-bindgen-0.2.100/src/lib.rs:1271:18
12: js_sys::global::get_global_object
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/js-sys-0.3.77/src/lib.rs:6082:29
13: core::ops::function::FnOnce::call_once
at /Users/williamseemueller/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
14: once_cell::unsync::Lazy<T,F>::force::{{closure}}
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:775:28
15: once_cell::unsync::OnceCell<T>::get_or_init::{{closure}}
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:591:57
16: once_cell::unsync::OnceCell<T>::get_or_try_init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:629:23
17: once_cell::unsync::OnceCell<T>::get_or_init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:591:19
18: once_cell::unsync::Lazy<T,F>::force
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:774:13
19: <once_cell::unsync::Lazy<T,F> as core::ops::deref::Deref>::deref
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:843:13
20: js_sys::global
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/js-sys-0.3.77/src/lib.rs:6051:12
21: wasm_bindgen_futures::queue::Queue::new
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasm-bindgen-futures-0.4.50/src/queue.rs:89:35
22: core::ops::function::FnOnce::call_once
at /Users/williamseemueller/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
23: once_cell::unsync::Lazy<T,F>::force::{{closure}}
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:775:28
24: once_cell::unsync::OnceCell<T>::get_or_init::{{closure}}
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:591:57
25: once_cell::unsync::OnceCell<T>::get_or_try_init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:629:23
26: once_cell::unsync::OnceCell<T>::get_or_init
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:591:19
27: once_cell::unsync::Lazy<T,F>::force
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:774:13
28: <once_cell::unsync::Lazy<T,F> as core::ops::deref::Deref>::deref
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs:843:13
29: wasm_bindgen_futures::queue::Queue::with
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasm-bindgen-futures-0.4.50/src/queue.rs:124:11
30: wasm_bindgen_futures::task::singlethread::Task::spawn
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasm-bindgen-futures-0.4.50/src/task/singlethread.rs:36:9
31: wasm_bindgen_futures::spawn_local
at /Users/williamseemueller/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasm-bindgen-futures-0.4.50/src/lib.rs:93:5
32: ais::ais::start_ais_stream
at ./src/ais.rs:22:5
33: ais::main
at ./src/main.rs:7:5
34: core::ops::function::FnOnce::call_once
at /Users/williamseemueller/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

Process finished with exit code 101



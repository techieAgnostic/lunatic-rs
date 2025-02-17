/*!
Framework for building Rust applications that run on [lunatic][1].

# Main concepts

The main abstraction in [lunatic][1] is a process. Contrary to operating system processes,
lunatic processes are lightweight and fast to spawn. They are designed for **massive**
concurrency. Like operating system processes, they are sandboxed. Each of them gets a separate
memory and they can't access the memory from other processes, not even through raw pointers.
If we want to exchange any information between process we need to do it through message passing.

This library makes processes feel native to the Rust language. They can be spawned from just a
function.

### Process types:

* **[`Process`]** - A process that can receive messages through a [`Mailbox`] or
    [`Protocol`](protocol::Protocol).
* **[`AbstractProcess`](AbstractProcess)** - Abstracts state management and message/request
    handling.
* **[`Supervisor`](supervisor::Supervisor)** - A process that can supervise others and re-spawn
    them if they fail.

### Linking

Processes can be linked together. This means that if one of them fails, all linked ones will be
killed too.

To spawn a linked process use the [`spawn_link`] function.

### Process configuration

Spawn functions have a variant that takes a [`ProcessConfig`]. This configuration can be used
to set a memory or CPU limit on the newly spawned process. It can also be used to control file
and network access permissions of processes.

### Setup

To run the example you will first need to download the lunatic runtime by following the
installation steps in [this repository][1]. The runtime is just single executable and runs on
Windows, macOS and Linux. If you have already Rust installed, you can get it with:
```bash
cargo install lunatic-runtime
```

[Lunatic][1] applications need to be compiled to [WebAssembly][2] before they can be executed by
the runtime. Rust has great support for WebAssembly and you can build a lunatic compatible
application just by passing the `--target=wasm32-wasi` flag to cargo, e.g:

```bash
# Add the WebAssembly target
rustup target add wasm32-wasi
# Build the app
cargo build --release --target=wasm32-wasi
```

This will generate a .wasm file in the `target/wasm32-wasi/release/` folder inside your project.
You can now run your application by passing the generated .wasm file to Lunatic, e.g:

```ignore
lunatic run target/wasm32-wasi/release/<name>.wasm
```

#### Better developer experience

To simplify developing, testing and running lunatic applications with cargo, you can add a
`.cargo/config.toml` file to your project with the following content:

```toml
[build]
target = "wasm32-wasi"

[target.wasm32-wasi]
runner = "lunatic run"
```

or inside of your Cargo project just run:
```
lunatic init
```
This will automatically crate the file above.

Now you can just use the commands you were already familiar with, such as `cargo run`, `cargo test`
and cargo is going to automatically build your project as a WebAssembly module and run it inside
`lunatic`.

### Testing

Lunatic provides a [`macro@test`] macro to run your tests inside processes. Check out the [`tests`][3]
directory for examples.

[1]: https://github.com/lunatic-solutions/lunatic
[2]: https://webassembly.org/
[3]: https://github.com/lunatic-solutions/rust-lib/tree/main/tests

*/

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod config;
mod error;
mod macros;
mod mailbox;
mod module;
mod process_local;
mod tag;

pub mod ap;
pub mod distributed;
pub mod function;
pub mod host;
pub mod metrics;
pub mod net;
pub mod panic;
pub mod protocol;
pub mod serializer;
pub mod supervisor;
#[doc(hidden)]
pub mod test;
pub mod time;

pub use ap::AbstractProcess;
pub use config::ProcessConfig;
pub use error::LunaticError;
pub use function::process::Process;
pub use lunatic_macros::{abstract_process, main};
pub use lunatic_test::test;
pub use mailbox::{Mailbox, MailboxResult};
pub use module::{Param, WasmModule};
#[doc(hidden)]
pub use process_local::statik::Key as __StaticProcessLocalInner;
pub use process_local::ProcessLocal;
pub use tag::Tag;

// temporary until merged,
// discussed here: https://github.com/lunatic-solutions/lunatic/pull/160
pub mod sqlite {
    #[link(wasm_import_module = "lunatic::sqlite")]
    extern "C" {
        pub fn open(path: *const u8, path_len: usize, conn_id: *mut u32) -> u64;
        pub fn query_prepare(
            conn_id: u64,
            query_str: *const u8,
            query_str_len: u32,
            len_ptr: *mut u32,
            resource_id: *mut u32,
        ) -> ();
        pub fn query_result_get(resource_id: u64, write_buf: *const u8, write_buf_len: u32) -> ();
        pub fn drop_query_result(resource_id: u64) -> ();
        pub fn execute(conn_id: u64, exec_str: *const u8, exec_str_len: u32) -> u32;
    }
}

/// Implemented for all resources held by the VM.
pub trait Resource {
    /// Returns process local resource ID.
    fn id(&self) -> u64;
    /// Turns process local resource ID into resource handle.
    ///
    /// # Safety
    ///
    /// Extra care needs to be taken when balancing host side resources. It's
    /// easy to create an invalid resource reference.
    unsafe fn from_id(id: u64) -> Self;
}

/// Suspends the current process for `duration` of time.
pub fn sleep(duration: std::time::Duration) {
    unsafe { host::api::process::sleep_ms(duration.as_millis() as u64) };
}

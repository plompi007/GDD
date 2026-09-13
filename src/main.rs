//! Desktop entry point. The real app-building logic lives in
//! `chainworks::app` (see that module's doc comment for why: Android needs
//! `#[bevy_main]` on a `main()` compiled into the library's cdylib, not the
//! bin crate) — this just calls into it.

fn main() {
    chainworks::app::main();
}

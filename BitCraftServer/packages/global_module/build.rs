#![allow(clippy::disallowed_macros)]

#[path = "../game/build_shared.rs"]
pub mod build_shared;

fn main() {
    build_shared::main_shared();
}

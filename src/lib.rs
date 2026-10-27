//! The generator for pmacs.io.
//!
//! The site is static. [`site::build`] renders `content/`, the vault in
//! `data/sky.toml` and the docs of every pmacs release into an output
//! directory; [`check::run`] verifies that every local link in it resolves;
//! [`serve::serve`] serves it on localhost.
//!
//! The docs are read from pmacs's repository at each release tag, as files;
//! this crate never depends on pmacs's crates (`docs/decisions.md`, D3).
#![forbid(unsafe_code)]

pub mod check;
pub mod config;
pub mod docs;
pub mod markdown;
pub mod pages;
pub mod serve;
pub mod site;
pub mod sky;

/// Every fallible step reports a message a person can act on.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

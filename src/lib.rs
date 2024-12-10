#![allow(unused)]

#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod loaders;
pub mod package;

#[cfg(test)]
mod tests;
#![feature(const_trait_impl)]

mod components;
pub use components::*;
mod presentation;
pub use presentation::*;

pub struct Base {
	pub a: i8,
}
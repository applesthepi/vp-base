#![feature(const_trait_impl)]
#![feature(extend_one)]

mod components;
pub use components::*;
mod presentation;
pub use presentation::*;
mod program_data;
pub use program_data::*;
mod processing;
pub use processing::*;

// TODO: OVERHAUL: make full grade engine with vpb being a complete subcomponent replacement and worker.

// TODO: replace all builders with structs

// #[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! gmuc {
	($arc:expr) => { unsafe {
		Arc::get_mut_unchecked(&mut $arc)
	}};
}

// #[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! gmuc_ref {
	($arc:expr) => { unsafe {
		Arc::get_mut_unchecked($arc)
	}};
}

// #[cfg(debug_assertions)]
// #[macro_export]
// macro_rules! gmuc {
// 	($arc:expr) => { unsafe {
// 		Arc::get_mut(&mut $arc).unwrap()
// 	}};
// }

// #[cfg(debug_assertions)]
// #[macro_export]
// macro_rules! gmuc_ref {
// 	($arc:expr) => {
// 		Arc::get_mut($arc).unwrap()
// 	};
// }
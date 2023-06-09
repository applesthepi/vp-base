#![feature(const_trait_impl)]

mod components;
pub use components::*;
mod presentation;
pub use presentation::*;

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
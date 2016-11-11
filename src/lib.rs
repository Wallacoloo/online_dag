// Requires feature-gate for returning impl Iterator
#![feature(conservative_impl_trait)]
// For efficient comparison of Rc's.
#![feature(ptr_eq)]

#[cfg(test)]
mod tests;

pub mod ondag;
pub mod rcdag;

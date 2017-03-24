// Requires feature-gate for returning impl Iterator
#![feature(conservative_impl_trait)]
// For efficient comparison of Rc's.
#![feature(ptr_eq)]
// For restricting access to struct members to specific modules.
#![feature(pub_restricted)]

#[cfg(test)]
mod tests;

pub mod iodagfull;
pub mod ondag;
pub mod poscostdag;
pub mod rcdag;

mod rcdagbase;

// Requires feature-gate for returning impl Iterator
#![feature(conservative_impl_trait)]

#[cfg(test)]
mod tests;

pub mod ondag;
pub mod rcdag;

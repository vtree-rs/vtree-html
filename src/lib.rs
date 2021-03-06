#![feature(plugin)]
#![feature(conservative_impl_trait)]
#![feature(untagged_unions)]
#![feature(proc_macro)]

extern crate vtree;
extern crate vtree_macros;
extern crate stdweb;

#[macro_use]
pub mod nodes;
pub mod context;

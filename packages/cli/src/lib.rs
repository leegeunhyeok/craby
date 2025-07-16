#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn run(options: options::RunOptions) {
    cli::run_impl(options);
}

mod cli;
mod options;

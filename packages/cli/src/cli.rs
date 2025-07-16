use crate::options;

pub fn run_impl(opts: options::RunOptions) {
    println!("verbose: {:#?}", opts.verbose);
    craby_cli::greeting();
    craby_codegen::greeting();
}

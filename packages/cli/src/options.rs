#[napi(object)]
pub struct RunOptions {
    pub verbose: Option<bool>,
}

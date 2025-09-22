use std::path::PathBuf;

use super::schema::Schema;

pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub schemas: Vec<Schema>,
}

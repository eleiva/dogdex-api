use diesel::deserialize::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize, Debug)]
pub struct Dog {
    pub id: i32,
    pub name: String,
    pub image_path: String,
}
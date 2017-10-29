use super::schema::{users,blobs};

#[derive(Queryable)]
pub struct User {
    pub username: String,
    pub phash: String
}

#[derive(Queryable)]
pub struct Blob {
    pub uuid: String,
    pub owner: String,
    pub value: String
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub phash: &'a str
}

#[derive(Insertable)]
#[table_name="blobs"]
pub struct NewBlob<'a> {
    pub uuid: &'a str,
    pub owner: &'a str,
    pub value: &'a str
}

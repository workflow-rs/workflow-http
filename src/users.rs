use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User{
    pub uid:String,
    pub username:String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BasicUser{
    pub uid:String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HJsonUser{
    pub user:String,
    pub pass:String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsersConfig{
    pub users:Vec<HJsonUser>
}

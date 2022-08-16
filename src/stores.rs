// use serde;
use std::collections::HashMap;
use std::{fs, result::Result, io::Error, fmt::Debug};
use std::path::Path;
use deser_hjson;
// use crate::HJsonUser;
use crate::users::UsersConfig;

pub trait Store: Send + Sync + 'static{
    fn authenticate(&self, username:String, password:String)->(bool, String);
}

pub struct MemoryStore{
    users:HashMap<String, String>
}

impl MemoryStore{
    pub fn new(users:HashMap<String, String>)->Self{
        MemoryStore{
            users
        }
    }
}

impl Store for MemoryStore{
    fn authenticate(&self, username:String, password:String)->(bool, String){
        if !self.users.contains_key(username.as_str()){
            return (false, "".to_string());
        }

        if let Some(pass) = self.users.get(username.as_str()){
            if pass.eq(&password){
                return (true, username);
            }
        }

        (false, "".to_string())
    }
}


pub fn from_hjson_file<P>(file:P)->Result<MemoryStore, Error>
where P:AsRef<Path>+Debug
{
    let contents = fs::read_to_string(&file)
        .expect(&format!("Unable to read file: {:?}",file));
    //println!("contents: {}", contents);

    let config:UsersConfig = deser_hjson::from_str(&contents).unwrap();
    //println!("config: {:?}", config);
    //println!("object? {}", data.is_object());

    //let data: Value = serde_hjson::from_str(&contents).unwrap();
    
    //let users = data.get_mut("users").unwrap().as_object().unwrap();
    //println!("data.users:{:?}", users);


    let mut result = HashMap::with_capacity(2);
    for u in config.users.iter(){
        result.insert(u.user.clone(), u.pass.clone());
        //println!("user: {}, {}", u.user, u.pass);
    }

    //result.insert("abc", "123");
    //result.insert("xyz", "xxx");

    Ok(MemoryStore::new(result))
}


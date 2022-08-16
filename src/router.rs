extern crate tide;
pub use async_trait::async_trait;
use tide::{
    Response, Server, Request, Next, Result, http::mime, Body, StatusCode, log,
    Middleware
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io;
use std::collections::BTreeMap;

use crate::UXHelper;
use crate::error::*;

pub struct Router<'a>{
    helper:UXHelper<'a>
}

impl Router<'static>{
    pub fn new(
        root:PathBuf,
        mount_map:HashMap<&'static str, &'static str>,
        source_map:HashMap<&'static str, &'static str>
    )->Self{
        let overrides:BTreeMap<String, String> = BTreeMap::new();
        Self::new_with_overrides(root, mount_map, source_map, overrides)
    }
    pub fn new_with_overrides(
        root:PathBuf,
        mount_map:HashMap<&'static str, &'static str>,
        source_map:HashMap<&'static str, &'static str>,
        overrides:BTreeMap<String, String>
    )->Self{
        let helper = UXHelper::new(
            root,
            mount_map,
            source_map,
            overrides
        );
        Router{
            helper
        }
    }

    pub async fn route<State:Clone + Send + Sync + 'static>(
        &self,
        req: Request<State>,
        next: Next<'_, State>,
        path:String,
        src:String,
        dest:String
    )->Result{
        let result = self.helper.parse_file(path, src, dest);
        match result{
            Err(Error::FileNotFound(_))=>{
                return Ok(next.run(req).await);
            },
            Err(Error::FileNotSupported(file))=>{
    
                match Body::from_file(&file).await {
                    Ok(body) => return Ok(Response::builder(StatusCode::Ok).body(body).build()),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        log::warn!("Router:File not found: {:?}", file);
                        return Ok(Response::new(StatusCode::NotFound));
                    }
                    Err(e) => return Err(e.into())
                };
            },
            Err(Error::IoError(io_error))=>{
                return Err(io_error.into());
            },
            Err(e) => {

                println!("router error: {:#?}", e);

                return Ok(Response::new(StatusCode::InternalServerError));

                // return Err(router::tide::Error)
                // pub struct StringError(pub String);
                // StringError
                //http_format_err!(500, "oh no")
                // Ok(())
            },
            Ok((_file, contents)) => {
                let res = Response::builder(StatusCode::Ok)
                    .body(contents)
                    //.body(contents)
                    //.header("Cache-Control", "max-age=604800")
                    .content_type(mime::JAVASCRIPT)
                    .build();
                return Ok(res)
            }
            
        }

    }

    pub fn init<State:Clone + Send + Sync + 'static>(self, app:&mut Server<State>){
        app.with(self);
    }
}

#[async_trait]
impl<State> Middleware<State> for Router<'static>
where
    State: Clone + Send + Sync + 'static,
{
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        
        let path = req.url().path().to_string();

        let key_value = self.helper.find_key_value(path.clone());
        //println!("key_value:{:?} for path:{}", key_value, path);
        if let Some((src, dest)) = key_value{
            return self.route(req, next, path.to_string(), src, dest).await;
        }

        Ok(next.run(req).await)
    }
}
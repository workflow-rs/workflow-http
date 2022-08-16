extern crate tide;
pub use async_trait::async_trait;
use crate::{Store, BasicUser};
use std::sync::Arc;
use std::fmt::Debug;
use tide::{
    Middleware, Request, Response, Next, Result,
    Server, StatusCode, http::Error
};

pub struct BasicAuthenticator{
    store:Arc<dyn Store>,
    header_name:String
}

impl BasicAuthenticator{
    pub fn new<S>(store:S)->Self
    where
        S:Store
    {
        Self {
            store:Arc::new(store),
            header_name:"Authorization".to_string()
        }
    }

    pub fn authenticate(&self, auth_param:String)->Result<Option<String>>{
        let bytes = base64::decode(auth_param);
        if bytes.is_err() {
            return Err(Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth param must be valid base64.",
            ));
        }

        let as_utf8 = String::from_utf8(bytes.unwrap());
        if as_utf8.is_err() {
            return Err(Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth param base64 must contain valid utf-8.",
            ));
        }

        let as_utf8 = as_utf8.unwrap();
        let parts: Vec<_> = as_utf8.split(':').collect();

        if parts.len() < 2 {
            return Ok(None);
        }

        let (username, password) = (parts[0], parts[1]);
        let (success,uid) = self.store
            .authenticate(username.to_string(), password.to_string());
        if success{
            return Ok(Some(uid));
        }

        Ok(None)
    }

    pub fn build_auth_response()->Result{
        let mut res = Response::new(401);
        res.append_header("WWW-Authenticate", "Basic");
        Ok(res)
    }

    pub fn init<State:Clone + Send + Sync + 'static+std::fmt::Debug>(self, app:&mut Server<State>){
        app.with(self);
    }
}

#[async_trait]
impl<'a, State> Middleware<State> for BasicAuthenticator
where
    State: Clone + Send + Sync + Debug + 'static,
{
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> Result {
        //println!("headers {:#?}", req);
        let headers_options = req.header(self.header_name.as_str());
        if headers_options.is_none(){
            return Self::build_auth_response();
        }
        let headers: Vec<_> = headers_options.unwrap().into_iter().collect();
        //if let Some(headers) = headers_options{
            //let headers:Vec<_> = headers.into_iter().collect();
            //println!("headers: {:?}", headers);
            let prefix = "Basic ";
            for header in headers{
                let header_str = header.to_string();
                //println!("header: {}", header_str);
                if !header_str.starts_with(prefix){
                    continue;
                }
                let user_pass = header_str[prefix.len()..].to_string();

                //println!("user_pass: {}", user_pass);
                let res = self.authenticate(user_pass);
                let res = match res{
                    Ok(a)=>a,
                    Err(e)=>{
                        //println!("authenticate:error: {:?}", e);
                        return Err(e);
                    }
                };

                //println!("authenticate:result: {:?}", res);

                match res {
                    Some(uid)=>{
                        println!("uid:{}", uid);
                        req.set_ext(BasicUser{
                            uid
                        });
                        break;
                    },
                    None=>{
                        return Self::build_auth_response();
                        //return Ok(Response::new(StatusCode::Forbidden));
                    }
                }
            }
        //}

        Ok(next.run(req).await)
    }
}

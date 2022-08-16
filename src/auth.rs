extern crate tide;
pub use async_trait::async_trait;
use tide::{
    Response, Server, Request, Next, Result,
    // Body,
    // StatusCode,
    // log, 
    http::Method,
    Middleware,
    // Error
};
use serde::{Deserialize, Serialize};
// use serde_json::json;
use std::sync::Arc;
use crate::{User, Store};



pub const USER_KEY:&str = "flow-user";


#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest{
    username:String,
    password:String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse<'a>{
    success:bool,
    code:&'a str,
    msg:String,
    uid:String
}

impl<'a> LoginResponse<'a>{
    
    pub fn new(success:bool, code:&'a str, msg:String, uid:String)->Self{
        LoginResponse{
            success,
            code,
            msg,
            uid
        }
    }

    pub fn success(uid:String)->Self{
        Self::new(true, "SUCCESS", "Login successful".to_string(), uid)
    }

    pub fn fail(code:&'a str, msg:String)->Self{
        Self::new(false, code, msg, "".to_string())
    }

    pub fn server_error()->Self{
        Self::new(false, "SERVER-ERROR", "Server error. Please try later.".to_string(), "".to_string())
    }

    pub fn already_logged_in()->Self{
        Self::fail("ALREADY-LOGGED-IN", "".to_string())
    }
    pub fn invalid_info(msg:String)->Self{
        Self::fail("INVALID-INFO", msg)
    }

}

impl<'a> Clone for LoginResponse<'a>{
    fn clone(&self)->Self{
        LoginResponse{
            success:self.success,
            code:self.code,
            msg:self.msg.clone(),
            uid:self.uid.clone()
        }
    }
}

impl<'a> From<LoginResponse<'a>> for Response{
    fn from(a:LoginResponse<'a>) -> Response {
        let res = serde_json::to_string(&a).unwrap();
        Response::from(res)
    }
}


pub struct Authenticator{
    auth_url:String,
    store:Arc<dyn Store>
}

impl<'a> Authenticator{
    pub fn new<S>(auth_url:&str, store:S)->Self
    where
        S: Store
    {
        Authenticator{
            auth_url:auth_url.to_string(),
            store:Arc::new(store)
        }
    }

    pub fn authenticate(&self, req:LoginRequest)->(bool, String){
        self.store.authenticate(req.username, req.password)
    }

    pub fn init<State:Clone + Send + Sync + 'static+std::fmt::Debug>(self, app:&mut Server<State>){
        let auth_url = self.auth_url.clone();
        app.with(self);

        app.at(&auth_url).post(|req: Request<State>| async move{

            let login_response = match req.ext::<LoginResponse>(){
                Some(login_response)=>login_response.clone(),
                None=>{
                    LoginResponse::server_error()
                }
            };
            
            println!("login_response: {:?}", login_response);
            Ok(login_response)
        });
    }
}


#[async_trait]
impl<'a, State> Middleware<State> for Authenticator
where
    State: Clone + Send + Sync + 'static,
{
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> Result {

        let path = req.url().path().to_string();
        let url = path.trim_end_matches('/');

        let is_login_req = req.method().eq(&Method::Post) && url.eq(&self.auth_url);
        //println!("request ### is_login_req {}", is_login_req);
        if !is_login_req{
            return Ok(next.run(req).await);
        }

        let session = req.session();
        if let Some(_user) = session.get::<User>(USER_KEY){
            req.set_ext(LoginResponse::already_logged_in());
        }else{
            let login_request:LoginRequest = match req.body_json::<LoginRequest>().await{
                Ok(r)=>r,
                Err(e)=>{
                    println!("login_request error: {:?}", e);
                    req.set_ext(LoginResponse::invalid_info(e.to_string()));
                    return Ok(next.run(req).await);
                }
            };

            println!("login_request: {:?}", login_request);

            //let username = login_request.username.clone();
            //let password = login_request.password.clone();
            let (success, uid) = self.authenticate(login_request);
            if success{
                req.set_ext(LoginResponse::success(uid));
            }else{
                req.set_ext(LoginResponse::invalid_info("".to_string()));
            }
        }

        Ok(next.run(req).await)
    }
}

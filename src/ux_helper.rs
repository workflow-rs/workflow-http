use std::collections::{HashMap, BTreeMap};
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::{fs, ffi::OsStr};
use regex::Regex;
use crate::error::*;
use std::convert::Into;

// #[derive(Debug)]
// pub enum UXHelperError{
//     FileDontExists,
//     UnsupportedFile(PathBuf),
//     IOError(io::Error)
// }

pub struct UXHelper<'a>{
    re:Regex,
    root: PathBuf,
    url_prefix:String,
    routes: Vec<(&'a str, &'a str)>,
    proxies: Vec<(&'a str, &'a str)>,
    overrides:BTreeMap<String, String>
}

impl<'a> UXHelper<'a>{
    pub fn new(
        root:PathBuf,
        mount_map:HashMap<&'a str, &'a str>,
        source_map:HashMap<&'a str, &'a str>,
        overrides:BTreeMap<String, String>
    )->Self{
        let mut mount = HashMap::with_capacity(100);
        mount.insert("flow-ux", "/node_modules/@aspectron/flow-ux");
        mount.insert("lit-html", "/node_modules/lit-html");
        mount.insert("lit", "/node_modules/lit");
        mount.insert("at_lit", "/node_modules/@lit");
        mount.insert("lit-element", "/node_modules/lit-element");
        mount.insert("sockjs", "/node_modules/sockjs-client/dist");
        mount.insert("webcomponents", "/node_modules/@webcomponents/webcomponentsjs");
    
        let mount_clone = mount.clone();
        let mut source = mount.clone();

        mount.extend(mount_map);
        source.extend(source_map);

        let mut routes:Vec<(&str, &str)> = Vec::with_capacity(mount.capacity());
        let mut proxies:Vec<(&str, &str)> = Vec::with_capacity(mount_clone.capacity());

        let mut m:Vec<(&&str, &&str)> = mount.iter().collect();
        m.sort_by_key(|a|{
            10000 - a.1.len()
        });
        for (key, mount_point) in m {
            if let Some(source_dir) = source.get(key){
                routes.push((*mount_point, *source_dir));
            }

            //println!("key: {}, mount_point:{}", key, mount_point);
            if let Some(k) = mount_clone.get(key){
                //println!("k: {}", k);
                proxies.push((*k, *mount_point));
            }else if let Some(k) = source.get(key){
                //println!("k2: {}", k);
                proxies.push((*k, *mount_point));
            }
        }

        //println!("routes: {:#?}", routes);
        //println!("proxies: {:#?}", proxies);

        let regexp = "(import|export)([^'\"]*)from[ ]{0,}['\"]([^'\"]*)['\"]";
        let re = Regex::new(regexp).unwrap();
        //let mut overrides_clone:BTreeMap<&'a str, String> = BTreeMap::new();

        //for (key, value) in overrides{
        //    overrides_clone.insert(key, value);
        //}

        UXHelper{
            re,
            root,
            routes,
            proxies,
            overrides,
            url_prefix:String::from("")
        }
    }

    pub fn set_url_prefix(&mut self, url_prefix:String){
        self.url_prefix = url_prefix;
    }

    pub fn find_key_value(&self, url:String)->Option<(String, String)>{
        for (key, val) in &self.routes {
            if url.starts_with(key){
                return Some((key.to_string(), val.to_string()));
            }
        }
        None
    }

    pub fn parse_file(
        &self,
        path:String,
        src:String,
        dest:String
    )->Result<(PathBuf, String)>{
        let path_clone = path.clone();
        match self.read_file(path, src, dest){
            Ok((file, contents))=>{
                let cont = self.parse_content(&file, &contents);
                //println!("parse_file:file:{:?}, cont:{:?}", file, cont);
                Ok((file, cont))
            }
            Err(e)=>{
                println!("parse_file:path:{}, error:{:?}", path_clone, e);
                Err(e)
            }
        }
    }

    pub fn parse_file_and_collect_links(
        &self,
        path:String,
        src:String,
        dest:String
    )->Result<(PathBuf, String, Vec<String>)>{
        match self.read_file(path, src, dest){
            Ok((file, contents))=>{
                //let dir = file.as_path().parent().unwrap();
                let (content, links) = self.parse_content_and_collect_links(&file, &contents);
                Ok((file, content, links))
            }
            Err(e)=>{
                Err(e)
            }
        }
    }

    pub fn collect_links(&self, file:&String)->Result<(String, Vec<String>)>{
        let file_buf = PathBuf::from(file);
        let contents = match fs::read_to_string(file){
            Ok(string)=>string,
            Err(e)=>{
                println!("fs::read_to_string error: {}", file);
                return Err(e.into())
            }
        };
        Ok(self.parse_content_and_collect_links(&file_buf, &contents))
    }

    pub fn read_file(
        &self,
        path:String,
        src:String,
        dest:String
    )->Result<(PathBuf, String)>{
        let path = path.replacen(&src, "", 1);
        let mut file_buf = self.root.clone()
            .join(dest.trim_matches('/'))
            .join(path.trim_matches('/'));

        let mut file = file_buf.as_path();

        let mut ext = match file.extension().and_then(OsStr::to_str){
            Some(ext)=>{
                ext.to_lowercase()
            },
            None=>{
                "".to_string()
            }
        };

        //let mut file_clone:Path;
        if ext.eq(""){
            if let Some(str) = file.to_str(){
                let mut str:Vec<&str> = str.trim_matches('/').split(MAIN_SEPARATOR).collect();
                if let Some(last) = str.pop(){
                    file_buf = file_buf.join(last);
                    file_buf.set_extension("js");
                    ext = "js".to_string();
                    file = file_buf.as_path();
                }
            }
        }

        //println!("#### file: {:?}, ext:{}", file, ext);

        if !file.exists(){
            // return Err(error_code!(ErrorCode::FileNotFound).with_file(file.to_path_buf()));
            return Err(Error::FileNotFound(file.to_path_buf()));
        }
        
        if !ext.eq("js") && !ext.eq("mjs"){
            return Err(Error::FileNotSupported(file.to_path_buf()));
            // return Err(error_code!(ErrorCode::FileNotSupported).with_file(file.to_path_buf()));
            // return Err(UXHelperError::UnsupportedFile(file.to_path_buf()));
        }
        
        let contents = fs::read_to_string(file)?;

        Ok((file.to_path_buf(), contents))
    }
    pub fn get_path_override(&self, _file:&PathBuf, key:&str)->Option<&String>{
        match self.overrides.get(key){
            Some(str)=>{
                if str.len() < 1{
                    None
                }else{
                    Some(str)
                }
            },
            None=>{
                None
            }
        }
    }
    fn replace(subject:&str, search:&str, replace:&str)->String{
        subject.replace(&format!("'{}'", search), &format!("'{}'", replace))
            .replace(&format!("\"{}\"", search), &format!("\"{}\"", replace))
    }
    pub fn parse_content(&self, file:&PathBuf, contents:&String)->String{
        return self.re.replace_all(contents, |cap: &regex::Captures| {
            //println!("contents:{:?}", contents);
            //println!("cap:{:?}", cap);
            if let Some(p) = self.get_path_override(file, &cap[3]){
                return Self::replace(&cap[0], &cap[3], &p);
            }
            let mut d = cap[3].to_string();

            if !d.ends_with(".js") && !d.ends_with(".mjs"){
                let mut str_vec:Vec<&str> = d.split("/").collect();
                if let Some(last) = str_vec.pop(){
                    d = format!("{}/{}.js", d, last);
                }
            }
            if d.starts_with("."){
                return Self::replace(&cap[0], &cap[3], &d);
            }
            if !d.starts_with("/"){
                d = format!("/{}", d);
            }
            let mut found = false;
            for (_url, proxy_url) in &self.proxies {
                if d.starts_with(proxy_url){
                    found = true;
                    break;
                }
            }
            if found {
                return Self::replace(&cap[0], &cap[3], &d);
            }
    
            if !d.starts_with("/node_modules"){
                d = format!("/node_modules{}", d);
            }

            for (url, proxy_url) in &self.proxies {
                if d.starts_with(url){
                    d = d.replace(url, proxy_url);
                }
            }
            
            Self::replace(&cap[0], &cap[3], &d)
        }).to_string();
    }

    pub fn parse_content_and_collect_links(&self, file:&PathBuf, contents:&String)->(String, Vec<String>){
        let mut links:Vec<String> = vec![];
        let content = self.re.replace_all(contents, |cap: &regex::Captures| {
            //println!("cap: {:#?}", cap);
            //return "".to_string();
            if let Some(p) = self.get_path_override(file, &cap[3]){
                links.push(p.clone());
                return Self::replace(&cap[0], &cap[3], &p);
            }
            let mut d = cap[3].to_string();

            if !d.ends_with(".js") && !d.ends_with(".mjs"){
                let mut str_vec:Vec<&str> = d.split("/").collect();
                if let Some(last) = str_vec.pop(){
                    d = format!("{}/{}.js", d, last);
                }
            }

            //relative urls
            if d.starts_with("."){
                //let link = dir.join(d.clone().replace("./", "")).to_str().unwrap().to_string();
                links.push(d.clone());
                return Self::replace(&cap[0], &cap[3], &d);
            }

            //make them absolute
            if !d.starts_with("/"){
                d = format!("/{}", d);
            }

            //if start with any proxy url (i.e  /flow/flow-ux.js)
            for (url, proxy_url) in &self.proxies {
                //println!("d: {}, proxy_url:{}", d, proxy_url);
                if d.starts_with(proxy_url){
                    //println!("found: d:{}\nproxy_url:{}\nurl:{}", d, proxy_url, url);
                    let real_url = d.replacen(proxy_url, url, 1);
                    //println!("proxy_url:{}\nreal-url:{}\nd:{}", proxy_url, real_url, d);
                    links.push(real_url.to_string());
                    return Self::replace(&cap[0], &cap[3], &format!("{}{}", self.url_prefix, &real_url));
                }
            }
    
            if !d.starts_with("/node_modules"){
                d = format!("/node_modules{}", d);
            }
            /*
            for (url, proxy_url) in &self.proxies {
                //println!("d: {},  url: {}, proxy_url:{}", d,  url, proxy_url);
                if d.starts_with(url){
                    d = d.replacen(url, proxy_url, 1);
                }
            }
            */
            links.push(d.clone());
            Self::replace(&cap[0], &cap[3], &format!("{}{}", self.url_prefix, &d))
        }).to_string();

        (content, links)
    }
}
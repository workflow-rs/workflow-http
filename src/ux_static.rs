use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::fs;
use regex::Regex;
use crate::error::*;
use std::collections::BTreeMap;

use crate::UXHelper;

pub struct UXStaticBuilder<'a>{
    helper:UXHelper<'a>,
    dest:PathBuf,
    root:PathBuf,
    node_modules_parent_dir:PathBuf,
    modules_src:String,
    modules_dest:String,
    dest_str:String,
    root_str:String,
    cache:Vec<String>,
    map_re:Regex
}

impl<'a> UXStaticBuilder<'a>{
    pub fn try_new(
        root:PathBuf,
        dest:PathBuf,
        node_modules_parent_dir:PathBuf,
        url_prefix:&str,
        mount_map:HashMap<&'a str, &'a str>,
        source_map:HashMap<&'a str, &'a str>
    )-> Result<Self> {
        let overrides: BTreeMap<String, String> = BTreeMap::new();
        Self::try_new_overrides(
            root, dest, 
            node_modules_parent_dir,
            url_prefix,
            mount_map,
            source_map,
            overrides
        )
    }
    pub fn try_new_overrides(
        root:PathBuf,
        dest:PathBuf,
        node_modules_parent_dir:PathBuf,
        url_prefix:&str,
        mount_map:HashMap<&'a str, &'a str>,
        source_map:HashMap<&'a str, &'a str>,
        overrides:BTreeMap<String, String>
    )-> Result<Self> {
        
        let mut helper = UXHelper::new(
            root.clone(),
            mount_map,
            source_map,
            overrides
        );
        helper.set_url_prefix(url_prefix.to_string());

        fs::create_dir(&dest)?;

        let root_str = root.to_str().unwrap().to_string();
        //let root_str = root.display().to_string();
        let dest_str = dest.to_str().unwrap().to_string();
        let mut modules_src = node_modules_parent_dir.to_str().unwrap().to_string();
        modules_src.push_str("/node_modules/");

        let mut modules_dest = dest_str.clone();
        modules_dest.push_str("/node_modules/");

        let static_builder = UXStaticBuilder{
            root,
            dest,
            node_modules_parent_dir,
            helper,
            cache:vec![],
            modules_src,
            modules_dest,
            root_str,
            dest_str,
            map_re: Regex::new(r"sourceMappingURL=(?P<file>.*\.js\.map)").unwrap()
        };
        
        Ok(static_builder)

    }

    pub fn parse(&mut self, path:String) -> Result<()> {
        if self.cache.contains(&path){
            return Ok(());
        }
        self.cache.push(path.clone());

        let key_value = self.helper.find_key_value(path.clone());
    
        if let Some((src, dest)) = key_value{
            println!("-- parsing:: {}", path);
            return self.parse_and_save(path, src, dest);
        }else{
            let mut path_normalized = path.clone();
            if !PathBuf::from(&path).as_path().is_absolute(){
                path_normalized = self.root.clone().join(&path)
                    .to_str().unwrap()
                    .to_string()
            }
            println!("-- getting-links::{}", path_normalized);
            
            match self.helper.collect_links(&path_normalized){
                Ok((content, links))=>{
                    //println!("content: {}", content);

                    //println!("links:{:#?}", links);
                    let dest = path_normalized.replacen(
                        &self.root_str,
                        &self.dest_str,
                        1
                    ).replacen(
                        &self.modules_src,
                        &self.modules_dest,
                        1
                    );

                    let map_file = self.get_map_file(&path_normalized, &content);

                    //println!(" -- save-file::{}", dest);
                    self.save_file(PathBuf::from(dest), content, map_file)?;
                    let path_buf = PathBuf::from(path_normalized);
                    let dir = path_buf.parent().unwrap();
                    self.digest_links(dir, links)?;
                },
                Err(e)=>{
                    println!("e: {:?}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn get_map_file(&self, path:&String, js_content:&String)->Option<(String, String)>{
        match self.map_re.captures(&js_content){
            Some(m)=>{
                //println!("m: {:#?}", &m["file"]);
                let file = PathBuf::from(path).with_file_name(&m["file"]);
                let _contents = match fs::read_to_string(&file){
                    Ok(str)=>{
                        return Some(((&m["file"]).to_string(), str))
                    },
                    Err(e)=>{
                        println!("Unable to read map file: {:?}", file);
                        println!("{:#?}", e);
                    }
                };
            },
            None=>{

            }
        }
        //js_content.
        //# sourceMappingURL=lit-element.js.map
        None
    }

    fn parse_and_save(
        &mut self,
        path:String,
        src:String,
        dest:String
    )->Result<()>{
        //println!("path:{}\nsrc:{}\ndest:{}", path, src, dest);
        let result = self.helper.parse_file_and_collect_links(
            path.clone(),
            src.clone(),
            dest.clone()
        );
        match result{
            Ok((_file, content, links))=>{
                let proxy_dir = PathBuf::from(path.clone());
                let proxy_dir = proxy_dir.parent().unwrap();
                let path = path.replacen(&src, "", 1);
                let dest_file = self.dest.clone()
                    .join(dest.trim_matches('/'))
                    .join(path.trim_matches('/'));

                //println!("dest_file: {:?}", dest_file);
                //println!("------- file: {:?} ------>", file);
                //println!("links: {:#?}", links);
                //println!("content:{}", content);
                self.save_file(dest_file, content, None)?;
                self.digest_links(&proxy_dir, links)?;
                
                return Ok(())
            },
            Err(e)=>{
                println!("parse_and_save:error {:?}", e);
                return Err(e);
            }
        }
    }

    fn digest_links(&mut self, dir:&Path, links:Vec<String>) -> Result<()> {
        for link in links.iter(){
            if link.starts_with("./"){
                let file = dir.clone().join(link.replace("./", ""));
                //println!("dep file: {:#?}", file);
                self.parse(file.to_str().unwrap().to_string())?;
            }else if link.starts_with("/"){
                if link.starts_with("/node_modules"){
                    let file = self.node_modules_parent_dir.clone().join(link.trim_matches('/'));
                    //println!("dep file: {:#?}", file);
                    self.parse(file.to_str().unwrap().to_string())?;
                }else{
                    let file = self.root.clone().join(link.trim_matches('/'));
                    //println!("dep file: {:#?}", file);
                    self.parse(file.to_str().unwrap().to_string())?;
                    //self.parse(link.clone());
                }
            }else if link.starts_with("../"){
                let file = dir.clone().join(link);
                //println!("dep file: {:#?}", file);
                self.parse(file.to_str().unwrap().to_string())?;
            }else{
                println!("link: {:#?}", link);
            }
        }
        Ok(())
    }

    fn save_file(&self, file:PathBuf, content:String, map_file:Option<(String, String)>) -> Result<()> {
        fs::create_dir_all(&file.parent().unwrap())?;
        fs::write(&file, content)?;
        if let Some((file_name, content)) = map_file{
            fs::write(&file.with_file_name(file_name), content)?;
        }
        Ok(())
    }
}
#![feature(impl_trait_in_assoc_type)]
use core::panic;
use std::cell::RefCell;
use std::collections::HashMap;

use std::f32::consts::E;
use std::vec::Vec;
use tokio::sync::broadcast;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::fs;
use anyhow::Error;
pub const DEFAULT_ADDR:&str="123.0.0.1:8080";
fn create_file(filename: &str) -> Result<File, Error> {
    let file = File::create(filename)?;
    Ok(file)
}

fn write_line_to_file(file: &mut File, line: &str) -> Result<(), Error> {
    writeln!(file, "{}", line)?;
    Ok(())
}

fn read_lines_from_file(filename: &str) -> Result<Vec<String>, Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

fn restore(file_name: &str) -> Result<RefCell<HashMap<String, String>>, std::io::Error> {
    let mut map = HashMap::new();

    let file = std::fs::File::open(file_name)?;
    let reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() == 3 && (parts[0] == "set" || parts[0] == "delete") {
                let key = parts[1].to_string();
                let value = parts[2].to_string();

                if parts[0] == "set" {
                    map.insert(key, value);
                } else {
                    map.remove(&key);
                }
            }
        }
    }
	

    Ok(RefCell::new(map))
}






 use volo::net::Address;
pub struct S {
	map: RefCell<HashMap<String,String>>,//key-value
	master_slave:bool,//is master
	master_address:String,//if slave 
	current_address:String,//decide the aof name
}
impl S {
	pub fn master_new(addr:&Address) -> S {
		S {
			map:RefCell::new(HashMap::<String,String>::new()),
			master_slave:true,
			master_address:String::from(" "),
			current_address:addr.to_string(),
		}
	}
	pub fn slave_new(addr:&Address,master_addr:String)->S{
		S {
			map:RefCell::new(HashMap::<String,String>::new()),
			master_slave:false,
			master_address:master_addr,
			current_address:addr.to_string(),
		}
	}
	 
}
fn get_aofname(s:String)->String{
	let mut name:String=s.clone();
	name.push_str(".txt");
	name
}
unsafe impl Send for S {}
unsafe impl Sync for S {}




fn exist(file_path:&String){//判断文件是否存在，如果不存在，则创建
    // let file_path = "path/to/your/file.txt"; // 替换为你要检查的文件路径
    match fs::metadata(file_path) {
        Ok(_) => {
            ()
        }
        Err(_) => {
			File::create(file_path);
            ()
        }
    }
}






#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
	//-----------------------------------------------------------------------
	//这是一个废弃的方案,但是删除会报错
	async fn server_get_item(&self, _req: volo_gen::volo::example::GetItemRequest) 
	-> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError>{
		let  resp = volo_gen::volo::example::GetItemResponse {opcode: 0, key: _req.key
			.clone(), value: " ".into(), success: false};
		Ok(resp)
	}
	//------------------------------------------------------------------------
	async fn get_item(&self, _req: volo_gen::volo::example::GetItemRequest) -> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError>{
		let mut masteraof:String;
		
		let mut selfaof=get_aofname(self.current_address.clone());
		
		exist(&(selfaof.clone()));
		if self.master_slave==false{//slave  同步数据
			masteraof=get_aofname(self.master_address.clone());
			exist(&(masteraof.clone()));
			match read_lines_from_file(&masteraof){
				Ok(info)=>{
					let mut file = File::create(&selfaof)?;
					for lines in info{
						write_line_to_file(&mut file,&lines)?;
					}
				}
				Err(Error)=>{
					panic!("error when copy aof");
				}
			}
		}

		//-> Result<RefCell<HashMap<String, String>>, std::io::Error>
		
		
		//let mut s=RefCell::new(HashMap::<String,String>::new());
		let mut map=self.map.borrow_mut();
		match restore(&selfaof){//调用重建函数，恢复数据
			Ok(info)=>{
				let inner=info.borrow();
				for (key,value) in inner.iter(){
					let k:String=key.into();
					let v:String=value.into();

					//panic!("key:{},value:{}",k,v);

					map.insert(k.clone(),v.clone());
				}
			}
			Err(Error)=>{panic!("error when restore");}
		}

		
		
		
		let mut resp = volo_gen::volo::example::GetItemResponse {opcode: 0, key: _req.key.clone(), value: " ".into(), success: false};
		// let mut map=self.map.borrow_mut();
		//每一个指令完成后写入aof
		let mut file = File::open(selfaof)?;
		
		
		match _req.opcode {
			0 => {//get
				let key: String = _req.key.into();
				if map.contains_key(&key){
                    resp.opcode=0;
                    resp.key=key.clone().into();
                    resp.value=map[&key].clone().into();
                    resp.success=true;
                }
                else{
                    resp.opcode=0;
                    resp.key=key.into();
                    resp.success=false;
                }
			},
			1 => {//set
				if self.master_slave{
					let key: String = _req.key.into();
					let value: String = _req.value.into();
					map.insert(key.clone(),value.clone());
        	        resp.opcode=1;
    	            resp.key=key.clone().into();
	                resp.value=value.clone().into();
					
					
					
					resp.success=true;
					
					//write_line_to_file(file: &mut File, line: &str)
					let mut r=String::from("set ");
					r.push_str(&(key.clone()));
					r.push_str(" ");
					r.push_str(&(value.clone()));

				//	panic!("r:{};self_addresss:{}",r,&selfaof);

					write_line_to_file(&mut file,&r);
				}
				else{
					panic!("# set command to a slave node #");//以后再改
				}
			},
			2 => {//del
				let key: String = _req.key.into();
				resp.opcode=2;
				resp.key=key.clone().into();
				if map.contains_key(&key){
                    map.remove(&key);
                    resp.success=true;
					let mut r=String::from("set ");
					r.push_str(&(key.clone()));
					write_line_to_file(&mut file,&r);

                }
                else{
                    resp.success=false;
                    
                }
			},
			3 => {//?
				//ping
				resp.opcode = 3;
				resp.value = _req.value.clone();
				resp.success = true;
			},
			
			
			_ => {
				tracing::info!("Invalic opcode");
			},
		}
		Ok(resp)
	}
}

fn get_string(num: u8) -> String {
	let mut num: u8 = num;
	let mut res = String::new();
	let mut pow: u8 = 1;
	while pow <= num {
		pow *= 10;
	}
	pow /= 10;
	while pow != 0 {
		res.push((num / pow + '0' as u8) as char);
		num = num % pow;
		pow = pow / 10;
	}
	res
}





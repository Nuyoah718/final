use lazy_static::lazy_static;
//use mini_redis::LogLayer;
use std::env;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::net::SocketAddr;
use std::io::BufReader;
// use volo_gen::volo::example::{GetItemResponse, get_item};
// use mini_redis::{S};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash1024(input: &str) -> i32 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash_value:i32 = hasher.finish() as i32;

    // 将哈希值映射到 0 到 1024 之间
    hash_value % 1025
}

static mut addr: String = String::new();

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        unsafe {
            let addrr: SocketAddr = addr.parse().unwrap();
            volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
                //.layer_outer(LogLayer)
                .address(addrr)
                .build()
        }
    };
}

pub struct Proxy {
    master_num: i32,          //有多少个主节点
    master_addr: Vec<String>, //主节点的地址
    slave_num: i32,           //有多少从节点
    slave_addr: Vec<String>,  //从节点的地址
}
impl Proxy {
    fn new(m_num: i32, m_addr: Vec<String>, s_num: i32, s_addr: Vec<String>) -> Proxy {
        Proxy {
            master_num: m_num,
            master_addr: m_addr,
            slave_addr: s_addr,
            slave_num: s_num,
        }
    }
}
fn hash_string_to_range(s: &str, range: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish() as usize;
    hash % range
}
//将字符串hash映射到0-1024，根据总的主节点数量进行分区
fn key_decide(key: String, pro: Proxy) -> String {
    let str_num:usize = hash_string_to_range(&key,1024);
    let n:usize = pro.master_num as usize;
    let k:usize = 1024 / n as usize;
    let mut index:usize = (str_num / k) as usize;
    if index == n {
        index -= 1;
    }
    //panic!("index:{},n:{},k:{},str_num:{}",index,n,k,str_num);
    let master_node_addr = (pro.master_addr[index]).clone(); //选择使用哪一个主节点
    master_node_addr
}
fn redirect(new_addr: String) -> String {
    //重定向，实际上不需要调用该函数...
    new_addr
}


fn create_proxy() -> Proxy {
    let mut _addr = String::new();
    let mut _is_slave;
    let mut master_num = 0;
    let mut slave_num = 0;
    let mut master_addr = Vec::new();
    let mut slave_addr = Vec::new();
    let file = std::fs::File::open("config").unwrap();
    let mut buf_reader = std::io::BufReader::new(file).lines();
    //
    while let Some(line) = buf_reader.next() {
        let line = line.unwrap();
        if line.len() == 0 {
            break;
        }
        let line: Vec<&str> = line.split(':').collect();
        if line[0] == "ip" {
            _addr = format!("{}:{}", line[1], line[2]);
        } else if line[0] == "is_slave" {
            _is_slave = line[1].to_string();
            if _is_slave == "true" {
                slave_num += 1;
                slave_addr.push(_addr.clone());
            } else {
                master_num += 1;
                master_addr.push(_addr.clone());
            }
        } else if line[0] == "<end>" {
        }
    }
    // let create_proxy = || ->Proxy {
    //     Proxy { master_num, master_addr, slave_num, slave_addr}
    // };

    //return
    Proxy {
        master_num,
        master_addr,
        slave_num,
        slave_addr,
    }
}

#[volo::main]
async fn main() {
    // 获取命令行参数，
    let args: Vec<String> = env::args().collect();
    unsafe {
        addr = args[1].clone();
    } //未完成，需要进一步处理//获取初始地址，后面决定是否转发（重定向）
      //决定是否连接proxy，不连接在最开始输 地址

    

    loop {
        let pro = create_proxy();
        print!("mini-redis>  ");
        let _ = io::stdout().flush();
        // 读入传入的命令
        let mut buf: String = String::new();
        let _ = std::io::stdin().read_line(&mut buf).unwrap();
        let buf: String = buf.trim().into();
        // 将读入的命令按照空格分裂成字符串向量
        let command: Vec<String> = parse_command(&buf);
        if command.len() == 0 {
            println!("error: The command is empty");
            continue;
        }
        let mut req = volo_gen::volo::example::GetItemRequest {
            opcode: 0,
            key: " ".into(),
            value: "pong".into(),
        };
        // 判断输入的命令，设置req

        if command[0] == "exit".to_string() {
            // 退出
            println!("Goodbye!");
            break;
        } else if command[0] == "get".to_string() {
            // get命令，则第二个参数是要搜索的key.

            if args[1] == "yes".to_string() {
                unsafe {
                    addr = key_decide(command[1].clone(), pro);
                }
            }

            req.opcode = 0;
            if command.len() < 2 {
                println!("Usage: get <key>");
                continue;
            }
            req.key = command[1].clone().into();
        } else if command[0] == "set".to_string() {
            // set命令，则第二个参数为要设置的key，第三个参数为要设置的值
            if args[1] == "yes".to_string() {
                unsafe {
                    addr = key_decide(command[1].clone(), pro);
                }
            }

            if command.len() < 3 {
                println!("Usage: set <key> <value>");
                continue;
            }
            req.opcode = 1;
            req.key = command[1].clone().into();
            req.value = command[2].clone().into();
        } else if command[0] == "del".to_string() {
            // del命令，则第二个参数为要删去的key
            if args[1] == "yes".to_string() {
                unsafe {
                    addr = key_decide(command[1].clone(), pro);
                }
            }
            if command.len() < 2 {
                println!("Usage: del <key>");
                continue;
            }
            req.opcode = 2;
            req.key = command[1].clone().into();
        } else if command[0] == "ping".to_string() {
            // ping命令
            req.opcode = 3;
            // 要是有message要返回message
            if command.len() > 1 {
                req.value = command[1].clone().into();
            }
        } else if command[0] == "redirect".to_string() {
            //转发
            println!("input new address please:");

            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(_) => println!("redirect to: {}", buffer),
                Err(err) => println!("Error reading line: {}", err),
            }
            unsafe { addr = buffer };
            println!("you have redirected");

        } else {
            println!("Can't find the command: {}", command[0]);
            continue;
        }
    

    // 将信息传递出去并得到返回的结果
    let resp = CLIENT.get_item(req).await;
    match resp {
        Ok(info) => {
            if info.opcode == 0 {
                if info.success {
                    println!("{}", info.value);
                } else {
                    println!("The key: {} is not in the database", info.key);
                }
            }
            if info.opcode == 1 {
                if info.success {
                    println!("Set/revalue success!");
                } 
            }
            if info.opcode == 2 {
                if info.success {
                    println!("Del success!");
                } else {
                    println!("The key: {} is not found in the database", info.key);
                }
            }
            if info.opcode == 3 {
                if info.success {
                    println!("{}", info.value.clone().to_string());
                } else {
                    println!("The connect is fail");
                }
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
    }
}

fn parse_command(buf: &String) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let v1: Vec<&str> = buf.split(" ").collect();
    for s in v1 {
        v.push(s.into());
    }
    v
}

fn get_num(v: &Vec<char>) -> i32 {
    let mut index = 0;
    let mut res = 0;
    while index < v.len() {
        res = res * 10 + (v[index] as i32 - '0' as i32);
        index += 1;
    }
    res
}

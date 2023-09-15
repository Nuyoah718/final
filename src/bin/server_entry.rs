use std::io::BufRead;
use std::process::Command;
use std::thread;
// use mini_redis::{DEFAULT_ADDR, S};

fn main() {
    let mut addr = String::new();
    let mut is_slave = String::new();
    let mut mas_addr = String::from("none");
    // let mut master_cnt = 0;
    // let mut slave_cnt = 0;

    //异步读取根目录下的文本文件
    let file = std::fs::File::open("config").unwrap();
    let mut buf_reader = std::io::BufReader::new(file).lines();
    while let Some(line) = buf_reader.next() {
        // 按照下面的格式读取文件中的内容
        // "
        // ip:127.0.0.1:8080
        // is_slave:ture
        // master:127.0.0.1:8081
        // "
        let line = line.unwrap();
        if line.len() == 0 {
            break;
        }
        let line: Vec<&str> = line.split(':').collect();
        if line[0] == "ip" {
            addr = format!("{}:{}", line[1], line[2]);
            println!("ip:{}", &addr);
        } else if line[0] == "is_slave" {
            is_slave = line[1].to_string();
            println!("is_slave:{}", &is_slave);
        } else if line[0] == "master" {
            if line.len() > 2 {
                mas_addr = format!("{}:{}", line[1], line[2]);
                println!("master:{}", &mas_addr);
            }
        } else if line[0] == "<end>" {
            //在新线程中调用start_cmd,并传入addr,is_slave,mas_addr作为命令行参数
            let cmd_str = format!(
                "cargo run --bin server {} {} {}",
                addr, is_slave, mas_addr
            );

            thread::spawn(move || {
                start_cmd(&cmd_str);
            });

            println!("end");
        }
    }
    // let _handler = thread::spawn(|| {
    //     start_cmd("echo hello");
    // });
    // _handler.join().unwrap();

    //sleep for 10 secs
    loop {} //keep the process alive
}

fn start_cmd(_string: &str) {
    Command::new("sh")
    .arg("-c")
    .arg(_string)
    .output()
    .expect("failed to execute process");
}

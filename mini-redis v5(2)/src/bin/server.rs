#![feature(impl_trait_in_assoc_type)]
use mini_redis::{DEFAULT_ADDR, S};
use std::net::SocketAddr;

#[volo::main]
async fn main() {
    let mut _addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
    let mut addr = volo::net::Address::from(_addr);
    let mut is_slave: bool = false;
    let mut mas_addr = volo::net::Address::from(DEFAULT_ADDR.parse::<SocketAddr>().unwrap());
    //读取命令行参数
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        addr = volo::net::Address::from(args[1].parse::<SocketAddr>().unwrap());
    }
    if args.len() > 2 {
        is_slave = args[2].parse::<bool>().unwrap();
    }
    if args.len() > 3 {
        mas_addr = volo::net::Address::from(match args[3].parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => DEFAULT_ADDR.parse::<SocketAddr>().unwrap(),
        });
    }
    //print三个参数
    // println!("ip:{}", &addr);
    // println!("is_slave:{}", &is_slave);
    // println!("master:{}", &mas_addr);
    if is_slave {
        volo_gen::volo::example::ItemServiceServer::new(S::slave_new(&addr, mas_addr.to_string()))
            //.layer_front(LogLayer)
            .run(addr)
            .await
            .unwrap();
    } else {
        volo_gen::volo::example::ItemServiceServer::new(S::master_new(&addr))
            //.layer_front(LogLayer)
            .run(addr)
            .await
            .unwrap();
    }

    // volo_gen::volo::example::ItemServiceServer::new(S::new())
    //     .layer_front(LogLayer)
    //     .run(addr)
    //     .await
    //     .unwrap();
}

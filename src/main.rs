use std::{env, io, process, thread};
use std::cmp::max;
use std::io::Write;
use std::net::{IpAddr, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::channel;
use clap::{Command, Arg, ArgAction};

const MAX_PORT: usize = 65535;
const MIN_PORT: usize = 1025;

fn scan<T: Into<IpAddr>>(min_port: usize, max_port: usize, index: usize, ip_addr: T, num_threads: usize, tx: mpsc::Sender<u16>) {
    let start_port = min_port + index;
    let ip_addr = ip_addr.into();
    let mut port = start_port as u16;
    loop {
        match TcpStream::connect((ip_addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }
        port = port + num_threads as u16;

        if (max_port - port as usize) <= num_threads {
            break;
        }
    }
}

fn main() {
    let mut handles = Vec::new();

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .about("A simple port scanner written in Rust")
        .version("1.0.0")
        .author("Yuwen 'Lucas' Tao")
        .arg(
            Arg::new("threads")
                .short('j')
                .long("threads")
                .action(ArgAction::Set)
                .help("Number of worker threads")
                .default_value("5")
                .required(false)
        )
        .arg(
            Arg::new("ip")
                .action(ArgAction::Set)
                .help("this is the ip address you want to scan")
                .required(true)
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .action(ArgAction::Set)
                .help("this is the range of ports you will scan")
                .required(false)
        )
        .get_matches();

    let ip_address = matches.get_one::<String>("ip").unwrap_or_else(|| { process::exit(0) }).clone();
    let threads = match matches.get_one::<String>("threads") {
        Some(threads) => threads.parse::<usize>().unwrap_or(5),
        None => 5
    };
    let ports = matches.get_one::<String>("port").unwrap_or(&format!("{}-{}", MIN_PORT, MAX_PORT)).clone();
    let min_and_max_ports = ports.split('-').map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>();

    println!("Scanning with {} threads..., ip_address: {}, scanning through port {} to port {}", threads, ip_address, min_and_max_ports[0], min_and_max_ports[1]);
    let min_port = min_and_max_ports[0];
    let max_port = min_and_max_ports[1];

    let (tx, rx) = channel();

    for i in 0..threads{
        let ip = ip_address.clone().parse::<IpAddr>().unwrap();
        let tx = tx.clone();
        let handle = thread::spawn(move || scan::<IpAddr>(min_port, max_port,i ,ip ,threads ,tx));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Done Scanning");
    drop(tx);

    for port in rx.iter() {
        println!("{}", port);
    }
}

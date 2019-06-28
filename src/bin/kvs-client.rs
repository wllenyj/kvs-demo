#![feature(async_await)]
extern crate log;
use log::LevelFilter;

use clap::AppSettings;
use kvs::{KvsClient, Result};
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;
use futures::executor;
use std::io::{stdin, stdout, Write};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "kvs-client",
    raw(global_settings = "&[\
                           AppSettings::DisableHelpSubcommand,\
                           AppSettings::VersionlessSubcommands]")
)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
    #[structopt(
        long,
        help = "Sets the server address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000",
        parse(try_from_str)
    )]
    addr: SocketAddr,
    #[structopt(short = "d", long = "debug")]
    debug: bool,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "Get the string value of a given string key")]
    Get {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
    },
    #[structopt(name = "set", about = "Set the value of a string key to a string")]
    Set {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
        #[structopt(name = "VALUE", help = "The string value of the key")]
        value: String,
    },
    #[structopt(name = "rm", about = "Remove a given string key")]
    Remove {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
    },
    #[structopt(name = "scan", about = "Scan all key string by the given regex")]
    Scan {
        #[structopt(name = "KEY", help = "A regex")]
        key: String,
    },
    #[structopt(name = "console", about = "console")]
    Console,
}

fn main() {
    let opt = Opt::from_args();
    if opt.debug {
        env_logger::builder().filter_level(LevelFilter::Debug).init();
    }
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    executor::block_on(async move {
        let mut client = KvsClient::connect(opt.addr).await?;
        //debug!("connect {:?}", client); 
        match opt.command {
            Command::Get{ key } => {
                if let Some(value) = client.get(key).await? {
                    println!("{}", value);
                }else {
                    println!("Key not found");
                }
            }
            Command::Set{ key, value } => {
                client.set(key, value).await?;
            }
            Command::Remove { key } => {
                client.remove(key).await?;
            }
            Command::Scan { key } => {
                let value = client.scan(key).await?;
                if value.len() > 0 {
                    for i in value.iter() {
                        println!("{}", i);
                    }
                }else {
                    println!("Key not found");
                }
            }
            Command::Console => {
                loop {
                    //println!("Enter command:");
                    print!("> ");
                    let _=stdout().flush();
                    let mut s=String::new();
                    stdin().read_line(&mut s).expect("Did not enter a correct string");
                    if let Some('\n')=s.chars().next_back() {
                        s.pop();
                    }
                    if let Some('\r')=s.chars().next_back() {
                        s.pop();
                    }

                    let mut v: Vec<&str> = s.split(' ').collect();
                    v.insert(0, "kvs-client");
                    let opt = Opt::from_iter(v);
                    match opt.command {
                        Command::Get{ key } => {
                            if let Some(value) = client.get(key).await? {
                                println!("{}", value);
                            }else {
                                println!("Key not found");
                            }
                        }
                        Command::Set{ key, value } => {
                            client.set(key, value).await?;
                        }
                        Command::Remove { key } => {
                            client.remove(key).await?;
                        }
                        Command::Scan { key } => {
                            let value = client.scan(key).await?;
                            if value.len() > 0 {
                                for i in value.iter() {
                                    println!("{}", i);
                                }
                            }else {
                                println!("Key not found");
                            }
                        }
                        _ => {
                            println!("quit");
                            break;
                        }
                    }
                    //print!("{}", s);
                }

            }
        };
        Ok(())
    })
}

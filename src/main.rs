#[macro_use]
extern crate serde_derive;

use clap::{App, Arg, SubCommand};

mod config;
mod ssh_util;

fn main() {
    let arg_match = App::new("sshTool").version("1.0")
        .author("Rookie. <gb880327@189.cn>")
        .about("ssh管理工具")
        .subcommand(SubCommand::with_name("list").about("查看服务器配置"))
        .subcommand(SubCommand::with_name("add").about("添加服务器")
            .arg(Arg::with_name("label").long("name").short("n").value_name("STRING").help("服务器名称"))
            .arg(Arg::with_name("host").long("host").short("h").value_name("STRING").help("服务器IP"))
            .arg(Arg::with_name("port").long("port").short("P").value_name("INT").help("服务器端口"))
            .arg(Arg::with_name("username").long("user").short("u").value_name("STRING").help("服务器用户名"))
            .arg(Arg::with_name("password").long("pwd").short("p").value_name("STRING").help("服务器密码"))
            .arg(Arg::with_name("private_key").long("key").short("k").value_name("STRING").help("服务器秘钥"))
        )
        .subcommand(SubCommand::with_name("rm").about("删除服务器")
            .arg(Arg::with_name("label").value_name("STRING").help("服务器名称"))
        ).subcommand(SubCommand::with_name("up").about("上传文件")
            .arg(Arg::with_name("file").long("file").short("f").value_name("STRING").help("本地文件路径"))
            .arg(Arg::with_name("target").long("target").short("t").value_name("STRING").help("远程文件路径"))
        ).subcommand(SubCommand::with_name("down").about("下载文件")
            .arg(Arg::with_name("file").long("file").short("f").value_name("STRING").help("远程文件路径"))
            .arg(Arg::with_name("target").long("target").short("t").value_name("STRING").help("本地文件路径"))
        )
        .get_matches();

    let is_add = arg_match.is_present("add");
    match ssh_util::SshUtil::new(is_add) {
        Ok(mut ssh_util) => {
            match ssh_util.exec(arg_match) {
                Ok(()) => {}
                Err(err) => println!("{}", err.to_string())
            }
        }
        Err(err) => println!("{}",  err.to_string())
    }
}

use std::env;
use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use dialoguer::Select;

use {
    std::{os::unix::process::CommandExt, process::Command},
};

use crate::config::{ConfingUtil, Server};
use std::fs::{OpenOptions};
use std::io::Write;

const SSHPASS: &'static [u8] = include_bytes!("sshpass");

pub struct SshUtil {
    config_util: ConfingUtil,
    path: String,
}

impl SshUtil {
    pub fn new(is_add: bool) -> Result<Self> {
        let mut config_path = env::current_exe()?;
        config_path.pop();
        let arg: String = config_path.to_str().unwrap_or("").parse()?;
        let path = match arg.contains(if cfg!(target_os = "windows") { "\\target\\debug" } else { "/target/debug" }) {
            true => Path::new(env!("CARGO_MANIFEST_DIR")),
            false => Path::new(&arg)
        };
        let sshpass_path = path.join("sshpass");
        let sshpass_path_str = sshpass_path.to_str().unwrap().to_string();
        if !sshpass_path.exists() {
            match OpenOptions::new().create(true).write(true).open(sshpass_path) {
                Ok(mut fs) => {
                    fs.write_all(SSHPASS)?;
                    Command::new("chmod").arg("+x").arg(sshpass_path_str).output()?;
                }
                Err(err) => panic!("{}", err.to_string())
            }
        }

        let config_util = ConfingUtil::get_config(path.join("sshConfig.toml").to_str().unwrap().parse()?, is_add)?;
        Ok(SshUtil { config_util, path: path.to_str().unwrap().to_string() })
    }

    fn choose_server(&self) -> Result<usize> {
        let items: Vec<String> = self.config_util.config.servers.iter().map(|x| x.label.clone()).collect();
        Ok(Select::new().items(&items).default(0).with_prompt("请选择需要登陆的服务器(默认选择第一个)").interact()?)
    }

    fn ssh_login(&self) {
        let index = self.choose_server().unwrap();
        let server = self.config_util.config.servers.get(index).unwrap();

        if server.identity_file.len() > 0 {
            let mut cmd = Command::new(format!("{}/sshpass", self.path));
            let cmd = cmd.arg("ssh").arg("-i").arg(&server.identity_file).arg("-p")
                .arg(&server.port.to_string()).arg(format!("{}@{}", server.username, server.host));
            cmd.exec();
        } else if server.password.len() > 0 {
            let mut cmd = Command::new(format!("{}/sshpass", self.path));
            let cmd = cmd.arg("-p").arg(&server.password)
                .arg("ssh").arg("-p")
                .arg(&server.port.to_string()).arg(format!("{}@{}", server.username, server.host));
            cmd.exec();
        } else {
            let mut cmd = Command::new("ssh");
            let cmd = cmd.arg("-p").arg(&server.port.to_string())
                .arg(format!("{}@{}", server.username, server.host));
            cmd.exec();
        }
    }

    /*
     * file_path: 本地文件路径
     * target_path: 远程文件路径
     */
    fn upload(&self, file_path: String, target_path: String) {
        let index = self.choose_server().unwrap();
        let server = self.config_util.config.servers.get(index).unwrap();

        if server.identity_file.len() > 0 {
            let mut cmd = Command::new("scp");
            let cmd = cmd.arg("-i").arg(&server.identity_file)
                .arg("-P").arg(&server.port.to_string())
                .arg(file_path)
                .arg(format!("{}@{}:{}", server.username, server.host, target_path));
            cmd.output().unwrap();
        } else if server.password.len() > 0 {
            let mut cmd = Command::new(format!("{}/sshpass", self.path));
            let cmd = cmd.arg("-p").arg(&server.password)
                .arg("scp").arg("-P")
                .arg(&server.port.to_string())
                .arg(file_path)
                .arg(format!("{}@{}:{}", server.username, server.host, target_path));
            cmd.output().unwrap();
        } else {
            let mut cmd = Command::new("scp");
            let cmd = cmd.arg("-P")
                .arg(&server.port.to_string())
                .arg(file_path)
                .arg(format!("{}@{}:{}", server.username, server.host, target_path));
            cmd.output().unwrap();
        }
        println!("文件上传完成！");
    }

    /*
     * file_path: 远程文件路径
     * target_path: 本地文件路径
     */
    fn download(&self, file_path: String, target_path: String) {
        let index = self.choose_server().unwrap();
        let server = self.config_util.config.servers.get(index).unwrap();
        if server.identity_file.len() > 0 {
            let mut cmd = Command::new("scp");
            let cmd = cmd
                .arg("-i").arg(&server.identity_file)
                .arg("-P").arg(&server.port.to_string())
                .arg("-r").arg("-C")
                .arg(format!("{}@{}:{}", server.username, server.host, file_path))
                .arg(target_path);
            cmd.output().unwrap();
        } else if server.password.len() > 0 {
            let mut cmd = Command::new(format!("{}/sshpass", self.path));
            let cmd = cmd.arg("-p").arg(&server.password)
                .arg("scp").arg("-P")
                .arg(&server.port.to_string())
                .arg("-r").arg("-C")
                .arg(format!("{}@{}:{}", server.username, server.host, file_path))
                .arg(target_path);
            cmd.output().unwrap();
        } else {
            let mut cmd = Command::new("scp");
            let cmd = cmd.arg("-P")
                .arg(&server.port.to_string())
                .arg("-r").arg("-C")
                .arg(format!("{}@{}:{}", server.username, server.host, file_path))
                .arg(target_path);
            cmd.output().unwrap();
        }
        println!("文件下载完成！");
    }

    pub fn exec(&mut self, matchs: ArgMatches) -> Result<()> {
        if matchs.is_present("list") {
            for (i, server) in self.config_util.config.servers.iter().enumerate() {
                println!("{}: {} - {}", i + 1, &server.label, &server.host);
            }
            return Ok(());
        }
        if matchs.is_present("add") {
            let add = matchs.subcommand_matches("add").unwrap();
            let label = add.value_of("label").expect("没有服务器名称").to_string();
            let host = add.value_of("host").expect("没有IP").to_string();
            let port = match add.value_of("port") {
                Some(port) => port.parse().expect("端口必须为数字"),
                None => 22
            };
            let username = add.value_of("username").expect("没有用户名").to_string();
            let password = match add.value_of("password") {
                Some(pwd) => pwd.to_string(),
                None => String::new()
            };
            let private_key = match add.value_of("private_key") {
                Some(key) => key.to_string(),
                None => String::new()
            };
            let identity_file = match add.value_of("identity_file") {
                Some(identity) => identity.to_string(),
                None => String::new()
            };
            return self.config_util.add(Server { label, host, port, username, password, private_key, identity_file });
        }
        if matchs.is_present("rm") {
            let rm = matchs.subcommand_matches("rm").unwrap();
            let label = rm.value_of("label").expect("没有服务器名称").to_string();
            return self.config_util.remove(label);
        }
        if matchs.is_present("up") {
            let up = matchs.subcommand_matches("up").unwrap();
            let file = up.value_of("file").expect("没有本地文件路径").to_string();
            let target = up.value_of("target").expect("没有远程文件路径").to_string();
            self.upload(file, target);
            return Ok(());
        }
        if matchs.is_present("down") {
            let up = matchs.subcommand_matches("down").unwrap();
            let file = up.value_of("file").expect("没有远程文件路径").to_string();
            let target = up.value_of("target").expect("没有本地文件路径").to_string();
            self.download(file, target);
            return Ok(());
        }
        self.ssh_login();
        Ok(())
    }
}
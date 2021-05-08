use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub label: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub private_key: String,
}


pub struct ConfingUtil {
    pub path: String,
    pub config: Config,
}

impl ConfingUtil {
    pub fn get_config(path: String, is_add: bool) -> Result<Self> {
        let config_path = path.clone();
        if !Path::new(&path).exists() {
            if !is_add {
                return Err(anyhow!("请先添加服务器配置！"));
            }
            Ok(ConfingUtil { path: config_path, config: Config { servers: vec![] } })
        } else {
            let config = match OpenOptions::new().read(true).open(path) {
                Ok(mut fs) => {
                    let mut config_str = String::new();
                    fs.read_to_string(&mut config_str)?;
                    let config: Config = toml::from_str(&config_str)?;
                    Ok(config)
                }
                Err(err) => Err(anyhow!(err.to_string()))
            }?;
            Ok(ConfingUtil { path: config_path, config })
        }
    }

    fn save(&self) -> Result<()> {
        let servers_str = toml::to_string_pretty(&self.config)?;
        match OpenOptions::new().write(true).create(true).append(false).truncate(true).open(&self.path) {
            Ok(mut fs) => {
                fs.write_all(servers_str.as_bytes())?;
                Ok(())
            }
            Err(err) => Err(anyhow!(err.to_string()))
        }
    }

    pub fn add(&mut self, server: Server) -> Result<()> {
        let exists = self.config.servers.iter().filter(|x| x.label.eq(&server.label)).count();
        if exists > 0 {
            return Err(anyhow!("服务器已存在！"));
        }
        self.config.servers.push(server);
        self.save()
    }

    pub fn remove(&mut self, server_name: String) -> Result<()> {
        let server = self.config.servers.iter().position(|x| x.label.eq(&server_name)).unwrap();
        if server as i32 >= 0 {
            self.config.servers.remove(server);
            return self.save();
        }
        Ok(())
    }
}
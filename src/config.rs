use clap::{Arg, Command};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub max_connections: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 24975,
            username: None,
            password: None,
            max_connections: 1000,
        }
    }
}

impl Config {
    pub fn from_args() -> Self {
        let matches = Command::new("RustProxy")
            .version("1.0")
            .author("RustProxy")
            .about("一个高性能的Rust HTTP代理服务器")
            .arg(
                Arg::new("ip")
                    .short('i')
                    .long("ip")
                    .value_name("IP")
                    .help("监听IP地址")
                    .default_value("0.0.0.0"),
            )
            .arg(
                Arg::new("port")
                    .short('p')
                    .long("port")
                    .value_name("PORT")
                    .help("监听端口")
                    .value_parser(clap::value_parser!(u16))
                    .default_value("24975"),
            )
            .arg(
                Arg::new("username")
                    .short('u')
                    .long("username")
                    .value_name("USERNAME")
                    .help("认证用户名"),
            )
            .arg(
                Arg::new("password")
                    .short('w')
                    .long("password")
                    .value_name("PASSWORD")
                    .help("认证密码"),
            )
            .arg(
                Arg::new("max_connections")
                    .short('c')
                    .long("max-connections")
                    .value_name("MAX_CONNECTIONS")
                    .help("最大并发连接数")
                    .value_parser(clap::value_parser!(usize))
                    .default_value("1000"),
            )
            .get_matches();

        let ip = matches
            .get_one::<String>("ip")
            .unwrap()
            .parse()
            .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));

        let port = *matches.get_one::<u16>("port").unwrap_or(&24975);
        let username = matches.get_one::<String>("username").cloned();
        let password = matches.get_one::<String>("password").cloned();
        let max_connections = *matches.get_one::<usize>("max_connections").unwrap_or(&1000);

        Config {
            ip,
            port,
            username,
            password,
            max_connections,
        }
    }

    pub fn auth_enabled(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }
}

use dialoguer::{Select, theme::ColorfulTheme};
use std::process::Command;

struct Server {
    name: &'static str,
    ip: &'static str,
    user: &'static str,
    password: Option<&'static str>,
}

fn main() {
    let servers = vec![
        Server { name: "Server 1", ip: "0.0.0.0", user: "a", password: Some("1234") },
        Server { name: "Server 2", ip: "0.0.0.1", user: "b", password: Some("1234") },
        Server { name: "Server 3", ip: "0.0.0.2", user: "c", password: Some("1234") },
    ];

    let items: Vec<String> = servers.iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {} ({})", i+1, s.name, s.ip))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("서버를 선택하세요")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    let server = &servers[selection];

    println!("선택된 서버: {} ({})", server.name, server.ip);

    if let Some(pass) = server.password {
        println!("PW init...");
        Command::new("expect")
            .arg("-c")
            .arg(format!(r#"
                spawn ssh {user}@{ip}
                expect "password:"
                send "{pass}\r"
                interact
            "#, user=server.user, ip=server.ip, pass=pass))
            .status()
            .unwrap();
    } else {
        println!("바로 접속");
        Command::new("ssh")
            .arg(format!("{}@{}", server.user, server.ip))
            .status()
            .unwrap();
    }
}
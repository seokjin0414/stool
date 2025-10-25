use stool_core::config::Server;
use stool_core::error::Result;
use stool_utils::{command, interactive};

pub fn connect(servers: &[Server]) -> Result<()> {
    let server_info = interactive::select_server(servers)?;

    let (user, ip, key_path, password) = match server_info {
        Some(info) => info,
        None => return Ok(()), // User cancelled
    };

    command::execute_ssh(&user, &ip, key_path.as_deref(), password.as_deref())
}

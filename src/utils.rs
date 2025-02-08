use rpassword::read_password;
use std::io::Write;

pub fn prompt_password() -> eyre::Result<String> {
    print!("Enter keystore password: ");
    std::io::stdout().flush()?;
    Ok(read_password()?)
}

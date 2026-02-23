use crate::environment::Environment;

pub fn create_env(name: String) -> anyhow::Result<()> {
    Environment::new(name)?;
    return Ok(());
}

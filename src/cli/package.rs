use crate::environment::Environment;

pub fn add_package(name: &String) {
    todo!();
}

pub fn remove_package(name: &String) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.remove_package(name)?;
    return Ok(());
}

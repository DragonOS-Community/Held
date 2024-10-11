use crate::application::mode::ModeKey;
use crate::application::Application;
use crate::errors::*;

pub fn exit(app: &mut Application) -> Result<()> {
    app.switch_mode(ModeKey::Exit);
    Ok(())
}

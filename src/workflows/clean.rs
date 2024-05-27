use crate::{BeansError, RunnerContext};

#[derive(Debug, Clone)]
pub struct CleanWorkflow {
    pub context: RunnerContext
}
impl CleanWorkflow {
    pub async fn wizard(_ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        todo!("please implement. clean action deletes temporary files")
    }
}
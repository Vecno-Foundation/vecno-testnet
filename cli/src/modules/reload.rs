use crate::imports::*;

#[derive(Default, Handler)]
#[help("Reload the web interface (used for testing)")]
pub struct Reload;

impl Reload {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, _argv: Vec<String>, _cmd: &str) -> Result<()> {
        // #[cfg(target_arch = "wasm32")]
        // workflow_dom::utils::window().location().reload().ok();

        let ctx = ctx.clone().downcast_arc::<VecnoCli>()?;
        tprintln!(ctx, "{}", style("reloading wallet ...").magenta());
        ctx.wallet().reload(true).await?;

        Ok(())
    }
}

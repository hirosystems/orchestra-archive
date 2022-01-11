use kompact::prelude::*;

#[derive(ComponentDefinition, Actor)]
pub struct UpdateStateExplorer {
    ctx: ComponentContext<Self>
}

impl UpdateStateExplorer {
    pub fn new() -> UpdateStateExplorer {
        UpdateStateExplorer {
            ctx: ComponentContext::uninitialised()
        }
    }
}

impl ComponentLifecycle for UpdateStateExplorer {
    fn on_start(&mut self) -> Handled {
        info!(self.ctx.log(), "Hello World!");
        self.ctx().system().shutdown_async();
        Handled::Ok
    }
}
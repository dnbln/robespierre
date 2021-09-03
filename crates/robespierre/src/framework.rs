use robespierre_models::channel::Message;

pub mod standard;

#[async_trait::async_trait]
pub trait Framework: Send + Sync + 'static {
    type Context: 'static;

    async fn handle(&self, ctx: Self::Context, message: &Message);
}
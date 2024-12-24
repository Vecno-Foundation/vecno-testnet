use async_channel::Sender;
use vecno_notify::notifier::Notify;
use vecno_rpc_core::Notification;

#[derive(Debug)]
pub struct ChannelNotify {
    sender: Sender<Notification>,
}

impl ChannelNotify {
    pub fn new(sender: Sender<Notification>) -> Self {
        Self { sender }
    }
}

impl Notify<Notification> for ChannelNotify {
    fn notify(&self, notification: Notification) -> vecno_notify::error::Result<()> {
        self.sender.try_send(notification)?;
        Ok(())
    }
}

use nats::Message;
use log::info;

pub(crate) fn run_surl(msg: Message) {
    info!("Handling message");
    msg.respond(msg.data.as_slice());
}
  
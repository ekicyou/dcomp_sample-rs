use crate::win_message_handler::*;
use crate::win_state::*;

pub trait Window: WinState + WinMessageHandler {}

use crate::win_message_handler::*;
use crate::win_state::*;
use ambassador::*;

#[delegatable_trait]
pub trait Window: WinState + WinMessageHandler {}

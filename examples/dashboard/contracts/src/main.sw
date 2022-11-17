contract;

use std::{
    address::Address,
    call_frames::contract_id,
    constants::BASE_ASSET_ID,
    identity::Identity,
    message::send_message,
    token::transfer,
};

abi Dashboard {
    fn trigger_transfer();
    fn trigger_transferout();
    fn trigger_messageout();
}

impl Dashboard for Contract {
    fn trigger_transfer() {
    }

    fn trigger_transferout() {
    }

    fn trigger_messageout() {
    }
}

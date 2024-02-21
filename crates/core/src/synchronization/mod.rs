use std::{sync::Arc, time::Duration};

use ethers::types::H160;
use helios::client::{Client, Database};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};
use starknet_crypto::FieldElement;
use tokio::sync::mpsc::Receiver;

use self::{l1_source::L1Source, l2_verifier::L2Verifier};

mod l1_source;
mod l2_verifier;

const CHANNEL_SIZE: usize = 64;

pub async fn synchronise(
    l1_client: Arc<Client<impl Database>>,
    l2_client: Arc<JsonRpcClient<HttpTransport>>,
    core_contract_addr: H160,
    poll_interval: Duration,
) -> Receiver<LocalSyncState> {
    let l1_updates = L1Source::spawn(l1_client, core_contract_addr, poll_interval);
    L2Verifier::spawn(l1_updates, l2_client).await
}

pub struct LocalSyncState {
    pub block_number: u64,
    pub state_root: FieldElement,
    pub type_: UpdateType,
}

pub enum UpdateType {
    /// Accepted on L1
    L1Synchronized,
    /// Verified from the last L1 update
    L2Verified
}

struct L1Update {
    block_number: u64,
    state_root: FieldElement,
}
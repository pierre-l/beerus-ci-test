use std::{sync::Arc, time::Duration};

use ethers::types::H160;
use eyre::Result;
use helios::client::{Client, Database};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, error};

use crate::client::{get_starknet_state_block_number, get_starknet_state_root};

use super::{L1Update, CHANNEL_SIZE};

#[derive(Clone)]
pub struct L1Source {}

impl L1Source {
    pub fn spawn(
        l1_client: Arc<Client<impl Database>>,
        core_contract_addr: H160,
        poll_interval: Duration,
    ) -> Receiver<L1Update> {
        let (state_tx, state_rx) = mpsc::channel(CHANNEL_SIZE);

        tokio::spawn(async move {
            if let Err(err) = Self::run(
                &l1_client,
                core_contract_addr,
                state_tx,
                poll_interval,
            )
            .await
            {
                error!("L1Source loop stopped. Cause: {}", err);
            }
        });

        state_rx
    }

    async fn run(
        l1_client: &Client<impl Database>,
        core_contract_addr: H160,
        state_tx: Sender<L1Update>,
        poll_interval: Duration,
    ) -> Result<()> {
        let mut last_update = 0;

        loop {
            let state_root =
                get_starknet_state_root(l1_client, core_contract_addr).await?;
            let new_l1_sync_block_number =
                get_starknet_state_block_number(l1_client, core_contract_addr)
                    .await?;

            if last_update < new_l1_sync_block_number {
                debug!("New L1 block: {}", new_l1_sync_block_number);
                state_tx
                    .send(L1Update {
                        block_number: new_l1_sync_block_number,
                        state_root,
                    })
                    .await?;
                last_update = new_l1_sync_block_number;
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

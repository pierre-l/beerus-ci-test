use std::sync::Arc;

use starknet::{core::types::{BlockId, BlockTag}, providers::{jsonrpc::HttpTransport, JsonRpcClient}};
use tokio::sync::mpsc::{self, Receiver, Sender};
use eyre::{eyre, Result};
use tracing::error;
use crate::block_hash::compute_block_hash;
use crate::l2_client::L2ClientExt;

use super::{L1Update, LocalSyncState, UpdateType, CHANNEL_SIZE};

pub struct L2Verifier;

impl L2Verifier {
    pub async fn spawn(
        l1_source: Receiver<L1Update>,
        l2_client: Arc<JsonRpcClient<HttpTransport>>,
    ) -> Receiver<LocalSyncState> {
        let (state_tx, state_rx) = mpsc::channel(CHANNEL_SIZE);

        tokio::spawn(async {
            if let Err(err) = Self::run(state_tx, l1_source, l2_client).await {
                error!("L2Verifier loop stopped. Cause: {}", err);
            }
        });

        state_rx
    }

    pub async fn run(
        state_tx: Sender<LocalSyncState>,
        mut l1_source: Receiver<L1Update>,
        l2_client: Arc<JsonRpcClient<HttpTransport>>,
    ) -> Result<()> {
        while let Some(l1_update) = l1_source.recv().await {
            state_tx.send(LocalSyncState {
                block_number: l1_update.block_number,
                state_root: l1_update.state_root,
                type_: UpdateType::L1Synchronized,
            }).await?;

            let l2_latest_block = l2_client
                .get_confirmed_block_with_tx_hashes(BlockId::Tag(
                    BlockTag::Latest,
                ))
                .await?;


            // TODO 550 Might be lagging behind here.
            // TODO 550 We might hash some blocks twice. LRU cache?
            Self::catch_up(&state_tx, &l2_client, l1_update.block_number, l2_latest_block.block_number).await?;
        }

        Ok(())
    }

    pub async fn catch_up(
        state_tx: &Sender<LocalSyncState>,
        l2_client: &JsonRpcClient<HttpTransport>,
        from: u64,
        to: u64,
    ) -> Result<()> {
        let mut synced_block = from;
        // Hash the block one by one.
        while synced_block <= to {
            let candidate_number = synced_block + 1;
            let id = BlockId::Number(candidate_number);
    
            let block = l2_client.get_confirmed_block_with_txs(id).await?;
            let events = l2_client.get_block_events(id).await?;
    
            // TODO 550 Check the parent hash too.
    
            let hash = compute_block_hash(&block, &events);
    
            if hash != block.block_hash {
                return Err(eyre!(
                    "Sync from proven root failed: inconsistent block hash at height {}",
                    candidate_number
                ));
            }
    
            synced_block = candidate_number;

            state_tx.send(LocalSyncState {
                block_number: block.block_number,
                state_root: block.block_hash,
                type_: UpdateType::L2Verified,
            }).await?;
        }

        Ok(())
    }
}
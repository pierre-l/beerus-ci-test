use eyre::{eyre, Result};
use starknet::{
    core::types::{
        BlockId, BlockWithTxHashes, BlockWithTxs, Event, EventFilter,
        MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs,
    },
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider},
};

/// TODO 550 Doc
pub trait L2ClientExt {
    // TODO 550 Doc
    fn get_confirmed_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<BlockWithTxHashes>> + Send;
    // TODO 550 Doc
    fn get_confirmed_block_with_txs(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<BlockWithTxs>> + Send;
    // TODO 550 Doc
    fn get_block_events(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Vec<Event>> + Send;
}

impl L2ClientExt for JsonRpcClient<HttpTransport> {
    async fn get_confirmed_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> Result<BlockWithTxHashes> {
        match self.get_block_with_tx_hashes(block_id).await {
            Ok(MaybePendingBlockWithTxHashes::Block(l2_block)) => Ok(l2_block),
            Ok(MaybePendingBlockWithTxHashes::PendingBlock(_)) => {
                Err(eyre!("expecting confirmed block, got pending"))
            }
            Err(e) => Err(eyre!("failed to fetch confirmed block: {e}")),
        }
    }

    async fn get_confirmed_block_with_txs(
        &self,
        block_id: BlockId,
    ) -> Result<BlockWithTxs> {
        match self.get_block_with_txs(block_id).await {
            Ok(MaybePendingBlockWithTxs::Block(l2_block)) => Ok(l2_block),
            Ok(MaybePendingBlockWithTxs::PendingBlock(_)) => {
                Err(eyre!("expecting confirmed block, got pending"))
            }
            Err(e) => Err(eyre!("failed to fetch confirmed block: {e}")),
        }
    }

    async fn get_block_events(&self, block_id: BlockId) -> Vec<Event> {
        let page_size = 1024;
        let mut events = vec![];
        let mut last_token = None;
        let mut continue_ = true;

        while continue_ {
            let events_page = self
                .get_events(
                    EventFilter {
                        from_block: Some(block_id),
                        to_block: Some(block_id),
                        address: None,
                        keys: None,
                    },
                    last_token,
                    page_size,
                )
                .await
                .unwrap();
            last_token = events_page.continuation_token;

            if last_token.is_none() {
                continue_ = false;
            }

            let mut new_events = events_page
                .events
                .into_iter()
                .map(|e| Event {
                    from_address: e.from_address,
                    keys: e.keys,
                    data: e.data,
                })
                .collect::<Vec<Event>>();
            events.append(&mut new_events);
        }

        events
    }
}

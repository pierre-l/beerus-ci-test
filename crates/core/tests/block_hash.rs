use beerus_core::{block_hash::compute_block_hash, l2_client::L2ClientExt};
use starknet::{
    core::types::{BlockId, BlockTag},
    providers::{jsonrpc::HttpTransport, JsonRpcClient},
};
use url::Url;

#[tokio::test]
async fn verify_latest_block_hash() {
    let rpc_client = {
        let rpc_url = std::env::var("STARKNET_RPC")
            .expect("Missing STARKNET_RPC env var");
        JsonRpcClient::new(HttpTransport::new(Url::parse(&rpc_url).unwrap()))
    };

    let block_id = BlockId::Tag(BlockTag::Latest);
    let block = rpc_client.get_confirmed_block_with_txs(block_id).await.unwrap();
    let events = rpc_client.get_block_events(block_id).await.unwrap();

    let expected = block.block_hash;
    assert_eq!(compute_block_hash(&block, &events), expected);
}

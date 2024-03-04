use std::{collections::HashSet, num::NonZeroU128, sync::Arc};

use blockifier::{
    block::{BlockInfo, GasPrices},
    context::{BlockContext, ChainInfo, FeeTokenAddresses, TransactionContext},
    execution::{
        common_hints::ExecutionMode,
        contract_class::ContractClass,
        entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext},
    },
    state::{
        cached_state::CommitmentStateDiff,
        state_api::{State as BlockifierState, StateReader, StateResult},
    },
    transaction::objects::{
        CommonAccountFields, DeprecatedTransactionInfo, TransactionInfo,
    },
    versioned_constants::VersionedConstants,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use starknet_api::{
    block::{BlockNumber as StarknetBlockNumber, BlockTimestamp},
    core::{
        ChainId as BlockifierChainId, ClassHash, CompiledClassHash,
        ContractAddress, EntryPointSelector, Nonce,
    },
    deprecated_contract_class::EntryPointType,
    hash::{StarkFelt, StarkHash},
    state::StorageKey as StarknetStorageKey,
    transaction::{
        Calldata, Fee, TransactionHash, TransactionSignature,
        TransactionVersion,
    },
};

use crate::gen;

pub mod err;
pub mod map;

use err::Error;

// https://github.com/eqlabs/pathfinder/blob/v0.11.0-rc0/crates/executor/src/call.rs#L16
pub fn exec(_url: &str, txn: gen::BroadcastedTxn) -> Result<(), Error> {
    #[allow(unused_variables)]
    let gen::BroadcastedTxn::BroadcastedInvokeTxn(gen::BroadcastedInvokeTxn(
        gen::InvokeTxn::InvokeTxnV0(gen::InvokeTxnV0 {
            calldata,
            contract_address,
            entry_point_selector,
            max_fee,
            signature,
            version,
            ..
        }),
    )) = txn
    else {
        return Err(Error::Custom("unexpected transaction type"));
    };

    let mut resources = ExecutionResources::default();

    let one = NonZeroU128::new(1).unwrap();
    let block_info = BlockInfo {
        block_number: StarknetBlockNumber(0),
        block_timestamp: BlockTimestamp(0),
        sequencer_address: ContractAddress(StarkHash::ZERO.try_into()?),
        gas_prices: GasPrices {
            eth_l1_gas_price: one,
            strk_l1_gas_price: one,
            eth_l1_data_gas_price: one,
            strk_l1_data_gas_price: one,
        },
        use_kzg_da: false,
    };

    let chain_info = ChainInfo {
        chain_id: BlockifierChainId("00".to_owned()),
        fee_token_addresses: FeeTokenAddresses {
            strk_fee_token_address: ContractAddress(
                StarkHash::ZERO.try_into()?,
            ),
            eth_fee_token_address: ContractAddress(StarkHash::ZERO.try_into()?),
        },
    };

    let versioned_constants = VersionedConstants::latest_constants();

    let block_context = BlockContext::new_unchecked(
        &block_info,
        &chain_info,
        versioned_constants,
    );

    let tx_info = TransactionInfo::Deprecated(DeprecatedTransactionInfo {
        common_fields: CommonAccountFields {
            transaction_hash: TransactionHash(StarkHash::ZERO),
            version: TransactionVersion(StarkHash::ZERO),
            signature: TransactionSignature(vec![StarkFelt::ZERO]),
            nonce: Nonce(StarkFelt::ZERO),
            sender_address: ContractAddress(StarkHash::ZERO.try_into()?),
            only_query: true,
        },
        max_fee: Fee(42),
    });

    let tx_context = Arc::new(TransactionContext { block_context, tx_info });

    let mut context = EntryPointExecutionContext::new(
        tx_context.clone(),
        ExecutionMode::Execute,
        /*limit_steps_by_resources=*/ false,
    )?;

    // TODO: convert and put necessary data from the input
    let call_entry_point = CallEntryPoint {
        class_hash: None,
        code_address: None,
        entry_point_type: EntryPointType::External,
        entry_point_selector: EntryPointSelector(StarkHash::ZERO),
        calldata: Calldata(Arc::new(vec![StarkFelt::ZERO])),
        storage_address: ContractAddress(StarkHash::ZERO.try_into()?),
        caller_address: ContractAddress(StarkHash::ZERO.try_into()?),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut proxy = StateProxy {};

    let call_info =
        call_entry_point.execute(&mut proxy, &mut resources, &mut context)?;

    println!("{call_info:?}");
    Ok(())
}

struct StateProxy {
    // TODO: add blocking client
}

impl StateReader for StateProxy {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StarknetStorageKey,
    ) -> StateResult<StarkFelt> {
        tracing::info!(?contract_address, ?key, "get_storage_at");
        todo!()
    }

    fn get_nonce_at(
        &mut self,
        contract_address: ContractAddress,
    ) -> StateResult<Nonce> {
        tracing::info!(?contract_address, "get_nonce_at");
        todo!()
    }

    fn get_class_hash_at(
        &mut self,
        contract_address: ContractAddress,
    ) -> StateResult<ClassHash> {
        tracing::info!(?contract_address, "get_class_hash_at");
        todo!()
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: ClassHash,
    ) -> StateResult<ContractClass> {
        tracing::info!(?class_hash, "get_compiled_contract_class");
        todo!()
    }

    fn get_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
    ) -> StateResult<CompiledClassHash> {
        tracing::info!(?class_hash, "get_compiled_class_hash");
        todo!()
    }
}

#[allow(unused_variables)]
impl BlockifierState for StateProxy {
    fn set_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StarknetStorageKey,
        value: StarkFelt,
    ) -> StateResult<()> {
        tracing::info!(?contract_address, ?key, ?value, "set_storage_at");
        Ok(())
    }

    fn increment_nonce(
        &mut self,
        contract_address: ContractAddress,
    ) -> StateResult<()> {
        tracing::info!(?contract_address, "increment_nonce");
        Ok(())
    }

    fn set_class_hash_at(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) -> StateResult<()> {
        tracing::info!(?contract_address, ?class_hash, "set_class_hash_at");
        Ok(())
    }

    fn set_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> StateResult<()> {
        tracing::info!(?class_hash, ?contract_class, "set_contract_class");
        Ok(())
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        tracing::info!(
            ?class_hash,
            ?compiled_class_hash,
            "set_compiled_class_hash"
        );
        Ok(())
    }

    fn to_state_diff(&mut self) -> CommitmentStateDiff {
        tracing::info!("to_state_diff");
        CommitmentStateDiff {
            storage_updates: Default::default(),
            address_to_nonce: Default::default(),
            address_to_class_hash: Default::default(),
            class_hash_to_compiled_class_hash: Default::default(),
        }
    }

    fn add_visited_pcs(&mut self, class_hash: ClassHash, pcs: &HashSet<usize>) {
        tracing::info!(?class_hash, pcs.len = pcs.len(), "add_visited_pcs");
    }
}
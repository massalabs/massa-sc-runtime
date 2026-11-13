use crate::as_execution::ASModule;
use crate::types::{Interface, InterfaceClone, Result};
use crate::{Compiler, CondomLimits, GasCosts, RuntimeModule};

use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PaginatedKeysCall {
    Current {
        prefix: Option<Vec<u8>>,
        start_key: Option<Vec<u8>>,
        count: u32,
    },
    ForAddress {
        address: String,
        prefix: Option<Vec<u8>>,
        start_key: Option<Vec<u8>>,
        count: u32,
    },
}

pub(crate) static PAGINATED_KEYS_FIXTURE: Mutex<BTreeSet<Vec<u8>>> = Mutex::new(BTreeSet::new());
pub(crate) static PAGINATED_KEYS_CALLS: Mutex<Vec<PaginatedKeysCall>> = Mutex::new(Vec::new());

fn select_paginated_keys(
    prefix: Option<&[u8]>,
    start_key: Option<&[u8]>,
    count: u32,
) -> BTreeSet<Vec<u8>> {
    PAGINATED_KEYS_FIXTURE
        .lock()
        .unwrap()
        .iter()
        .filter(|key| prefix.is_none_or(|prefix| key.starts_with(prefix)))
        .filter(|key| start_key.is_none_or(|start_key| key.as_slice() > start_key))
        .take(count as usize)
        .cloned()
        .collect()
}

#[derive(Clone)]
pub(crate) struct TestInterface;

impl InterfaceClone for TestInterface {
    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(self.clone())
    }
}

impl Interface for TestInterface {
    fn increment_recursion_counter(&self) -> Result<()> {
        Ok(())
    }

    fn decrement_recursion_counter(&self) -> Result<()> {
        Ok(())
    }

    fn init_call(&self, _address: &str, _raw_coins: u64) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn get_interface_version(&self) -> Result<u32> {
        Ok(0)
    }

    fn finish_call(&self) -> Result<()> {
        Ok(())
    }

    fn transfer_coins(&self, _to_address: &str, _raw_amount: u64) -> Result<()> {
        Ok(())
    }

    fn transfer_coins_for(
        &self,
        _from_address: &str,
        _to_address: &str,
        _raw_amount: u64,
    ) -> Result<()> {
        Ok(())
    }

    fn address_from_public_key(&self, _public_key: &str) -> Result<String> {
        Ok("".to_string())
    }

    fn generate_event(&self, _event: String) -> Result<()> {
        Ok(())
    }

    fn get_call_stack(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn get_current_period(&self) -> Result<u64> {
        Ok(0)
    }

    fn get_current_thread(&self) -> Result<u8> {
        Ok(0)
    }

    fn get_module(&self, bytecode: &[u8], gas_limit: u64) -> Result<RuntimeModule> {
        let as_module = ASModule::new(
            bytecode,
            gas_limit,
            GasCosts::default(),
            Compiler::CL,
            CondomLimits::default(),
        )
        .unwrap();
        let module = RuntimeModule::ASModule(as_module);
        Ok(module)
    }

    fn get_tmp_module(&self, bytecode: &[u8], gas_limit: u64) -> Result<RuntimeModule> {
        let as_module = ASModule::new(
            bytecode,
            gas_limit,
            GasCosts::default(),
            Compiler::SP,
            CondomLimits::default(),
        )
        .unwrap();
        let module = RuntimeModule::ASModule(as_module);
        Ok(module)
    }

    fn get_owned_addresses(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn has_data(&self, _key: &[u8]) -> Result<bool> {
        Ok(false)
    }

    fn has_data_for(&self, _address: &str, _key: &[u8]) -> Result<bool> {
        Ok(false)
    }

    fn hash(&self, _data: &[u8]) -> Result<[u8; 32]> {
        Ok([0; 32])
    }

    fn raw_append_data(&self, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_append_data_for(&self, _address: &str, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_delete_data(&self, _key: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_delete_data_for(&self, _address: &str, _key: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_get_data(&self, _key: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn raw_get_data_for(&self, _address: &str, _key: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn raw_set_data(&self, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_set_data_for(&self, _address: &str, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn signature_verify(&self, _data: &[u8], _signature: &str, _public_key: &str) -> Result<bool> {
        Ok(false)
    }

    fn unsafe_random(&self) -> Result<i64> {
        Ok(0)
    }

    fn get_balance(&self) -> Result<u64> {
        Ok(0)
    }

    fn get_balance_for(&self, _address: &str) -> Result<u64> {
        Ok(0)
    }

    fn raw_set_bytecode(&self, _bytecode: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_set_bytecode_for(&self, _address: &str, _bytecode: &[u8]) -> Result<()> {
        Ok(())
    }

    fn get_call_coins(&self) -> Result<u64> {
        Ok(0)
    }

    fn create_module(&self, module: &[u8]) -> Result<String> {
        if module.len() > 32 {
            let mut bytes = Vec::new();
            for item in module.iter().take(32) {
                bytes.push(item);
            }
        }
        Ok("sc_address".to_string())
    }

    /// Print function for examples
    fn print(&self, _message: &str) -> Result<()> {
        Ok(())
    }

    fn send_message(
        &self,
        target_address: &str,
        target_handler: &str,
        validity_start: (u64, u8),
        validity_end: (u64, u8),
        max_gas: u64,
        raw_fee: u64,
        coins: u64,
        data: &[u8],
        filter: Option<(&str, Option<&[u8]>)>,
    ) -> Result<()> {
        print!(
            "Send message to {} with target handler {}, validity start {:?}, validity end {:?}, max gas {}, raw fee {}, coins {}, data {:?}, filter {:?}",
            target_address, target_handler, validity_start, validity_end, max_gas, raw_fee, coins, data, filter
        );
        Ok(())
    }

    fn get_op_keys(&self, prefix: Option<&[u8]>) -> Result<Vec<Vec<u8>>> {
        let data = vec![
            vec![0, 1, 2, 3, 4, 5, 6, 11],
            vec![127, 128],
            vec![254, 255],
        ];

        match prefix {
            Some(prefix) => {
                // dummy implementation
                let filtered = data.iter().filter(|k| k[0] == prefix[0]).cloned().collect();
                Ok(filtered)
            }
            None => Ok(data),
        }
    }

    fn op_entry_exists(&self, key: &[u8]) -> Result<bool> {
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        Ok(ds.contains_key(key))
    }

    fn get_op_data(&self, key: &[u8]) -> Result<Vec<u8>> {
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        Ok(ds.get(key).cloned().unwrap_or(vec![]))
    }

    fn unsafe_random_f64(&self) -> Result<f64> {
        let ret: f64 = rand::random();
        Ok(ret)
    }

    fn get_time(&self) -> Result<u64> {
        // Using chrono as a test dummy implementation to make sure the ABI is
        // called correctly Note that Massa node implementation uses the
        // time of the context slot in order to ensure determinism, not
        // the UTC time
        Ok(chrono::offset::Utc::now()
            .timestamp_millis()
            .try_into()
            .unwrap())
    }

    // Sha256 hash data
    fn hash_sha256(&self, bytes: &[u8]) -> Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let hash = hasher.finalize().into();
        Ok(hash)
    }

    // Keccak256 hash data
    fn hash_keccak256(&self, bytes: &[u8]) -> Result<[u8; 32]> {
        let mut hasher = Keccak256::new();
        hasher.update(bytes);
        let hash = hasher.finalize().into();

        Ok(hash)
    }

    fn get_keys(&self, _prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>> {
        todo!()
    }

    fn get_keys_for(&self, _address: &str, _prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>> {
        todo!()
    }

    fn get_keys_paginated(
        &self,
        prefix: Option<&[u8]>,
        start_key: Option<&[u8]>,
        count: u32,
    ) -> Result<BTreeSet<Vec<u8>>> {
        PAGINATED_KEYS_CALLS
            .lock()
            .unwrap()
            .push(PaginatedKeysCall::Current {
                prefix: prefix.map(<[u8]>::to_vec),
                start_key: start_key.map(<[u8]>::to_vec),
                count,
            });
        Ok(select_paginated_keys(prefix, start_key, count))
    }

    fn get_keys_for_paginated(
        &self,
        address: &str,
        prefix: Option<&[u8]>,
        start_key: Option<&[u8]>,
        count: u32,
    ) -> Result<BTreeSet<Vec<u8>>> {
        PAGINATED_KEYS_CALLS
            .lock()
            .unwrap()
            .push(PaginatedKeysCall::ForAddress {
                address: address.to_owned(),
                prefix: prefix.map(<[u8]>::to_vec),
                start_key: start_key.map(<[u8]>::to_vec),
                count,
            });
        Ok(select_paginated_keys(prefix, start_key, count))
    }

    fn raw_get_bytecode(&self) -> Result<Vec<u8>> {
        todo!()
    }

    fn raw_get_bytecode_for(&self, _address: &str) -> Result<Vec<u8>> {
        todo!()
    }

    fn caller_has_write_access(&self) -> Result<bool> {
        Ok(true)
    }

    fn evm_signature_verify(
        &self,
        _message: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool> {
        Ok(true)
    }

    fn evm_get_address_from_pubkey(&self, _public_key: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn evm_get_pubkey_from_signature(&self, _hash: &[u8], _signature: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn is_address_eoa(&self, _address: &str) -> Result<bool> {
        Ok(true)
    }

    fn validate_address(&self, _address: &str) -> Result<bool> {
        Ok(true)
    }

    fn get_origin_operation_id(&self) -> Result<Option<String>> {
        Ok(Some(String::new()))
    }

    fn chain_id(&self) -> Result<u64> {
        Ok(7)
    }

    fn save_gas_remaining_before_subexecution(&self, _gas_used_until: u64) {}

    fn get_deferred_call_quote(
        &self,
        _target_slot: (u64, u8),
        _gas_limit: u64,
        _params_size: u64,
    ) -> Result<(bool, u64)> {
        Ok((true, 0))
    }

    fn deferred_call_register(
        &self,
        _target_addr: &str,
        _target_func: &str,
        _target_slot: (u64, u8),
        _max_gas: u64,
        _params: &[u8],
        _coins: u64,
    ) -> Result<String> {
        Ok("sample_test_id".to_string())
    }

    fn deferred_call_exists(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }

    fn deferred_call_cancel(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "gas_calibration")]
pub mod tests_gas_calibration;
#[cfg(not(feature = "gas_calibration"))]
pub mod tests_runtime;

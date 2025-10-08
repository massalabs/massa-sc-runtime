use crate::as_execution::ASModule;
use crate::types::{Interface, InterfaceClone, Result};
use crate::{Compiler, CondomLimits, GasCosts, RuntimeModule};

use massa_proto_rs::massa::model::v1::*;
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct TestInterface;

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

    fn init_call_wasmv1(&self, _address: &str, _raw_coins: NativeAmount) -> Result<Vec<u8>> {
        Ok(vec![])
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

    fn transfer_coins_wasmv1(
        &self,
        _to_address: String,
        _raw_amount: NativeAmount,
        from_address: Option<String>,
    ) -> Result<()> {
        match from_address {
            Some(_from_address) => {}
            None => {}
        }
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

    fn get_current_slot(&self) -> Result<Slot> {
        Ok(Slot {
            period: 0,
            thread: 0,
        })
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

    fn ds_entry_exists_wasmv1(&self, _key: &[u8], address: Option<String>) -> Result<bool> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(false)
    }

    fn hash(&self, _data: &[u8]) -> Result<[u8; 32]> {
        Ok([0; 32])
    }

    fn get_bytecode_wasmv1(&self, _address: Option<String>) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn get_ds_keys_wasmv1(
        &self,
        _prefix: &[u8],
        address: Option<String>,
    ) -> Result<BTreeSet<Vec<u8>>> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(BTreeSet::new())
    }

    fn raw_append_data(&self, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_append_data_for(&self, _address: &str, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn append_ds_value_wasmv1(
        &self,
        _key: &[u8],
        _value: &[u8],
        address: Option<String>,
    ) -> Result<()> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(())
    }

    fn raw_delete_data(&self, _key: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_delete_data_for(&self, _address: &str, _key: &[u8]) -> Result<()> {
        Ok(())
    }

    fn delete_ds_entry_wasmv1(&self, _key: &[u8], address: Option<String>) -> Result<()> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(())
    }

    fn raw_get_data(&self, _key: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn raw_get_data_for(&self, _address: &str, _key: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn get_ds_value_wasmv1(&self, _key: &[u8], address: Option<String>) -> Result<Vec<u8>> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(vec![])
    }

    fn raw_set_data(&self, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_set_data_for(&self, _address: &str, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn set_ds_value_wasmv1(
        &self,
        _key: &[u8],
        _value: &[u8],
        address: Option<String>,
    ) -> Result<()> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(())
    }

    fn signature_verify(&self, _data: &[u8], _signature: &str, _public_key: &str) -> Result<bool> {
        Ok(false)
    }

    fn unsafe_random(&self) -> Result<i64> {
        Ok(0)
    }

    fn unsafe_random_wasmv1(&self, num_bytes: u64) -> Result<Vec<u8>> {
        let bytes = vec![0; num_bytes as usize];
        Ok(bytes)
    }

    fn get_balance(&self) -> Result<u64> {
        Ok(0)
    }

    fn get_balance_for(&self, _address: &str) -> Result<u64> {
        Ok(0)
    }

    fn get_balance_wasmv1(&self, address: Option<String>) -> Result<NativeAmount> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(NativeAmount {
            mantissa: 0,
            scale: 1,
        })
    }

    fn raw_set_bytecode(&self, _bytecode: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_set_bytecode_for(&self, _address: &str, _bytecode: &[u8]) -> Result<()> {
        Ok(())
    }

    fn set_bytecode_wasmv1(&self, _bytecode: &[u8], address: Option<String>) -> Result<()> {
        match address {
            Some(_address) => {}
            None => {}
        }
        Ok(())
    }

    /// Generate a smart contract event
    fn generate_event_wasmv1(&self, _event: Vec<u8>) -> Result<()> {
        Ok(())
    }

    fn get_call_coins(&self) -> Result<u64> {
        Ok(0)
    }

    fn get_call_coins_wasmv1(&self) -> Result<NativeAmount> {
        Ok(NativeAmount {
            mantissa: 0,
            scale: 0,
        })
    }

    fn create_module(&self, module: &[u8]) -> Result<String> {
        if module.len() > 32 {
            let mut bytes = Vec::new();
            for item in module.iter().take(32) {
                bytes.push(item);
            }
        } else {
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

    fn get_op_keys_wasmv1(&self, _prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
        Ok(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 11],
            vec![127, 128],
            vec![254, 255],
        ])
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

    /// Returns the blake3 hash of the given bytes
    fn hash_blake3(&self, _bytes: &[u8]) -> Result<[u8; 32]> {
        Ok([0u8; 32])
    }

    fn native_amount_from_str_wasmv1(&self, amount: &str) -> Result<NativeAmount> {
        let mantissa = amount.parse::<u64>().unwrap();
        Ok(NativeAmount { mantissa, scale: 0 })
    }

    fn native_amount_to_string_wasmv1(&self, _amount: &NativeAmount) -> Result<String> {
        Ok("string amount".to_string())
    }

    fn check_native_amount_wasmv1(&self, _amount: &NativeAmount) -> Result<bool> {
        Ok(true)
    }

    fn add_native_amount_wasmv1(
        &self,
        _amount1: &NativeAmount,
        _amount2: &NativeAmount,
    ) -> Result<NativeAmount> {
        Ok(NativeAmount {
            mantissa: 100,
            scale: 0,
        })
    }

    fn sub_native_amount_wasmv1(
        &self,
        _amount1: &NativeAmount,
        _amount2: &NativeAmount,
    ) -> Result<NativeAmount> {
        Ok(NativeAmount {
            mantissa: 100,
            scale: 0,
        })
    }

    fn scalar_mul_native_amount_wasmv1(
        &self,
        _amount: &NativeAmount,
        _factor: u64,
    ) -> Result<NativeAmount> {
        Ok(NativeAmount {
            mantissa: 100,
            scale: 0,
        })
    }

    fn scalar_div_rem_native_amount_wasmv1(
        &self,
        _dividend: &NativeAmount,
        _divisor: u64,
    ) -> Result<(NativeAmount, NativeAmount)> {
        Ok((
            NativeAmount {
                mantissa: 100,
                scale: 0,
            },
            NativeAmount {
                mantissa: 0,
                scale: 0,
            },
        ))
    }

    fn div_rem_native_amount_wasmv1(
        &self,
        _dividend: &NativeAmount,
        _divisor: &NativeAmount,
    ) -> Result<(u64, NativeAmount)> {
        Ok((
            1,
            NativeAmount {
                mantissa: 0,
                scale: 0,
            },
        ))
    }

    fn check_address_wasmv1(&self, _to_check: &str) -> Result<bool> {
        Ok(true)
    }

    fn check_pubkey_wasmv1(&self, _to_check: &str) -> Result<bool> {
        Ok(true)
    }

    fn check_signature_wasmv1(&self, _to_check: &str) -> Result<bool> {
        Ok(true)
    }

    fn get_address_category_wasmv1(&self, _to_check: &str) -> Result<AddressCategory> {
        Ok(AddressCategory::ScAddress)
    }

    fn get_address_version_wasmv1(&self, _address: &str) -> Result<u64> {
        Ok(1)
    }

    fn get_pubkey_version_wasmv1(&self, _pubkey: &str) -> Result<u64> {
        Ok(1)
    }

    fn get_signature_version_wasmv1(&self, _signature: &str) -> Result<u64> {
        Ok(1)
    }

    fn checked_add_native_time_wasmv1(
        &self,
        time1: &NativeTime,
        time2: &NativeTime,
    ) -> Result<NativeTime> {
        Ok(NativeTime {
            milliseconds: time1.milliseconds + time2.milliseconds,
        })
    }

    fn checked_sub_native_time_wasmv1(
        &self,
        _time1: &NativeTime,
        _time2: &NativeTime,
    ) -> Result<NativeTime> {
        Ok(NativeTime { milliseconds: 0 })
    }

    fn checked_mul_native_time_wasmv1(
        &self,
        _time: &NativeTime,
        _factor: u64,
    ) -> Result<NativeTime> {
        Ok(NativeTime { milliseconds: 0 })
    }

    fn checked_scalar_div_native_time_wasmv1(
        &self,
        _dividend: &NativeTime,
        _divisor: u64,
    ) -> Result<(NativeTime, NativeTime)> {
        Ok((
            NativeTime { milliseconds: 0 },
            NativeTime { milliseconds: 0 },
        ))
    }

    fn checked_div_native_time_wasmv1(
        &self,
        _dividend: &NativeTime,
        _divisor: &NativeTime,
    ) -> Result<(u64, NativeTime)> {
        Ok((1, NativeTime { milliseconds: 0 }))
    }

    fn base58_check_to_bytes_wasmv1(&self, s: &str) -> Result<Vec<u8>> {
        Ok(s.to_string().into_bytes())
    }

    fn bytes_to_base58_check_wasmv1(&self, _bytes: &[u8]) -> String {
        "bs58checked".to_string()
    }

    fn compare_address_wasmv1(&self, left: &str, right: &str) -> Result<ComparisonResult> {
        let left: String = left.chars().skip(1).collect();
        let right: String = right.chars().skip(1).collect();
        let left = bs58::decode(left).with_check(None).into_vec();
        let right = bs58::decode(right).with_check(None).into_vec();

        match (left, right) {
            (Ok(left), Ok(right)) => {
                let res = match left.cmp(&right) {
                    std::cmp::Ordering::Less => ComparisonResult::Lower,
                    std::cmp::Ordering::Equal => ComparisonResult::Equal,
                    std::cmp::Ordering::Greater => ComparisonResult::Greater,
                };
                Ok(res)
            }
            _ => Ok(ComparisonResult::Equal),
        }
    }

    fn compare_native_amount_wasmv1(
        &self,
        left: &NativeAmount,
        right: &NativeAmount,
    ) -> Result<ComparisonResult> {
        let res = match left.mantissa.cmp(&right.mantissa) {
            std::cmp::Ordering::Less => ComparisonResult::Lower,
            std::cmp::Ordering::Equal => ComparisonResult::Equal,
            std::cmp::Ordering::Greater => ComparisonResult::Greater,
        };
        Ok(res)
    }

    fn compare_native_time_wasmv1(
        &self,
        left: &NativeTime,
        right: &NativeTime,
    ) -> Result<ComparisonResult> {
        let res = match left.milliseconds.cmp(&right.milliseconds) {
            std::cmp::Ordering::Less => ComparisonResult::Lower,
            std::cmp::Ordering::Equal => ComparisonResult::Equal,
            std::cmp::Ordering::Greater => ComparisonResult::Greater,
        };
        Ok(res)
    }

    fn compare_pub_key_wasmv1(&self, left: &str, right: &str) -> Result<ComparisonResult> {
        let left: String = left.chars().skip(1).collect();
        let right: String = right.chars().skip(1).collect();
        let left = bs58::decode(left).with_check(None).into_vec();
        let right = bs58::decode(right).with_check(None).into_vec();

        match (left, right) {
            (Ok(left), Ok(right)) => {
                let res = match left.cmp(&right) {
                    std::cmp::Ordering::Less => ComparisonResult::Lower,
                    std::cmp::Ordering::Equal => ComparisonResult::Equal,
                    std::cmp::Ordering::Greater => ComparisonResult::Greater,
                };
                Ok(res)
            }
            _ => Ok(ComparisonResult::Equal),
        }
    }

    fn get_keys(&self, _prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>> {
        todo!()
    }

    fn get_keys_for(&self, _address: &str, _prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>> {
        todo!()
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

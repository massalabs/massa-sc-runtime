use crate::as_execution::ASModule;
use crate::types::{Interface, InterfaceClone};
use crate::{Compiler, GasCosts, RuntimeModule};

use anyhow::Result;
use massa_proto_rs::massa::model::v1::{NativeHash, Slot};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::collections::BTreeMap;

#[derive(Clone)]
struct TestInterface;

impl InterfaceClone for TestInterface {
    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(self.clone())
    }
}

impl Interface for TestInterface {
    fn init_call(&self, address: &str, raw_coins: u64) -> Result<Vec<u8>> {
        println!("Init call to {}, with {} coins", address, raw_coins);
        Ok(vec![])
    }

    fn finish_call(&self) -> Result<()> {
        println!("Finish call");
        Ok(())
    }

    fn get_balance(&self) -> Result<u64> {
        println!("Get balance");
        Ok(0)
    }

    fn transfer_coins(&self, to_address: &str, raw_amount: u64) -> Result<()> {
        println!("Transfer {} coins to {}", raw_amount, to_address);
        Ok(())
    }

    fn transfer_coins_for(
        &self,
        from_address: &str,
        to_address: &str,
        raw_amount: u64,
    ) -> Result<()> {
        println!(
            "Transfer {} coins from {} to {}",
            raw_amount, from_address, to_address
        );
        Ok(())
    }

    fn address_from_public_key(&self, public_key: &str) -> Result<String> {
        println!("Address from public key {}", public_key);
        Ok("".to_string())
    }

    fn generate_event(&self, event: String) -> Result<()> {
        println!("Generate event {}", event);
        Ok(())
    }

    fn get_call_stack(&self) -> Result<Vec<String>> {
        println!("Get call stack");
        Ok(vec![])
    }

    fn get_current_period(&self) -> Result<u64> {
        println!("Get current period");
        Ok(0)
    }

    fn get_current_thread(&self) -> Result<u8> {
        println!("Get current thread");
        Ok(0)
    }

    fn get_current_slot(&self) -> Result<Slot> {
        println!("Get current slot");
        Ok(Slot {
            period: 0,
            thread: 0,
        })
    }

    fn get_module(&self, bytecode: &[u8], limit: u64) -> Result<RuntimeModule> {
        println!("Get module");
        let as_module =
            ASModule::new(bytecode, limit, GasCosts::default(), Compiler::SP)?;
        let module = RuntimeModule::ASModule(as_module);
        Ok(module)
    }

    fn get_owned_addresses(&self) -> Result<Vec<String>> {
        println!("Get owned addresses");
        Ok(vec![])
    }

    fn has_data(&self, key: &[u8]) -> Result<bool> {
        println!("Has data at {:?}", key);
        Ok(false)
    }

    fn has_data_for(&self, address: &str, key: &[u8]) -> Result<bool> {
        println!("Has data for {} at {:?}", address, key);
        Ok(false)
    }

    fn hash(&self, data: &[u8]) -> Result<[u8; 32]> {
        println!("Hash with data {:?}", data);
        Ok([0; 32])
    }

    fn raw_append_data(&self, key: &[u8], value: &[u8]) -> Result<()> {
        println!("Raw append data at {:?} with value {:?}", key, value);
        Ok(())
    }

    fn raw_append_data_for(
        &self,
        address: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        println!(
            "Raw append data for {} at {:?} with value {:?}",
            address, key, value
        );
        Ok(())
    }

    fn raw_delete_data(&self, key: &[u8]) -> Result<()> {
        println!("Raw delete data at {:?}", key);
        Ok(())
    }

    fn raw_delete_data_for(&self, address: &str, key: &[u8]) -> Result<()> {
        println!("Raw delete data for {} at {:?}", address, key);
        Ok(())
    }

    fn raw_get_data_for(&self, address: &str, key: &[u8]) -> Result<Vec<u8>> {
        println!("Raw get data for {} at {:?}", address, key);
        Ok(vec![])
    }

    fn raw_set_data(&self, key: &[u8], value: &[u8]) -> Result<()> {
        println!("Raw set data at {:?} with value {:?}", key, value);
        Ok(())
    }

    fn raw_set_data_for(
        &self,
        address: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        println!(
            "Raw set data for {} at {:?} with value {:?}",
            address, key, value
        );
        Ok(())
    }

    fn signature_verify(
        &self,
        data: &[u8],
        signature: &str,
        public_key: &str,
    ) -> Result<bool> {
        println!(
            "Signature verify with data {:?}, signature {} and public key {}",
            data, signature, public_key
        );
        Ok(false)
    }

    fn unsafe_random(&self) -> Result<i64> {
        println!("Unsafe random");
        Ok(0)
    }

    fn get_balance_for(&self, _address: &str) -> Result<u64> {
        println!("Get balance for");
        Ok(0)
    }

    fn raw_set_bytecode_for(
        &self,
        address: &str,
        bytecode: &[u8],
    ) -> Result<()> {
        println!(
            "Raw set bytecode for {} with bytecode {:?}",
            address, bytecode
        );
        Ok(())
    }

    fn raw_set_bytecode(&self, bytecode: &[u8]) -> Result<()> {
        println!("Raw set bytecode with bytecode {:?}", bytecode);
        Ok(())
    }

    fn print(&self, message: &str) -> Result<()> {
        println!("Print {}", message);
        Ok(())
    }

    fn raw_get_data(&self, key: &[u8]) -> Result<Vec<u8>> {
        println!("Raw get data at {:?}", key);
        Ok(vec![])
    }

    fn get_call_coins(&self) -> Result<u64> {
        println!("Get call coins");
        Ok(0)
    }

    fn create_module(&self, module: &[u8]) -> Result<String> {
        println!("Create module with module {:?}", module);
        Ok("".to_string())
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

    fn get_op_keys(&self) -> Result<Vec<Vec<u8>>> {
        println!("Get op keys");
        Ok(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 11],
            vec![127, 128],
            vec![254, 255],
        ])
    }

    fn has_op_key(&self, key: &[u8]) -> Result<bool> {
        println!("Has op key at {:?}", key);
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        Ok(ds.contains_key(key))
    }

    fn get_op_data(&self, key: &[u8]) -> Result<Vec<u8>> {
        println!("Get op data at {:?}", key);
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        Ok(ds.get(key).cloned().unwrap_or(vec![]))
    }

    fn unsafe_random_f64(&self) -> Result<f64> {
        println!("Unsafe random f64");
        let ret: f64 = rand::random();
        Ok(ret)
    }

    fn get_time(&self) -> Result<u64> {
        println!("Get time");
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
        println!("Hash sha256 with bytes {:?}", bytes);
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let hash = hasher.finalize().into();
        Ok(hash)
    }

    // Keccak256 hash data
    fn hash_keccak256(&self, bytes: &[u8]) -> Result<[u8; 32]> {
        println!("Hash keccak256 with bytes {:?}", bytes);
        let mut hasher = Keccak256::new();
        hasher.update(bytes);
        let hash = hasher.finalize().into();

        Ok(hash)
    }

    /// Returns the native hash of the given bytes
    fn native_hash(&self, bytes: &[u8]) -> Result<NativeHash> {
        println!("Native hash with bytes {:?}", bytes);

        let hash_bytes = [0u8; 32];
        let hash = NativeHash { version: 0, content: hash_bytes.to_vec() };

        Ok(hash)
    }
}

#[cfg(feature = "gas_calibration")]
pub mod tests_gas_calibration;
#[cfg(not(feature = "gas_calibration"))]
pub mod tests_runtime;

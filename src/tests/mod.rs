use crate::as_execution::ASModule;
use crate::types::{Interface, InterfaceClone};
use crate::{Compiler, GasCosts, RuntimeModule};

use anyhow::{bail, Result};
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::{collections::BTreeMap, str::FromStr};

pub type Ledger = std::collections::BTreeMap<String, Vec<u8>>; // Bytecode instead of String

#[derive(Clone)]
struct TestInterface(Arc<Mutex<Ledger>>);

impl InterfaceClone for TestInterface {
    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(self.clone())
    }
}

impl Interface for TestInterface {
    fn init_call(&self, address: &str, _raw_coins: u64) -> Result<Vec<u8>> {
        let data = self.0.lock().clone();
        match data.get::<String>(&address.to_string()) {
            Some(module) => Ok(module.clone()),
            _ => bail!("Cannot find module for address {}", address),
        }
    }

    fn finish_call(&self) -> Result<()> {
        Ok(())
    }

    fn transfer_coins(
        &self,
        _from_address: &str,
        _to_address: &str,
        _raw_amount: u64,
    ) -> Result<()> {
        Ok(())
    }

    fn address_from_public_key(&self, public_key: &str) -> Result<String> {
        Ok(public_key.to_string())
    }

    fn generate_event(&self, event: String) -> Result<()> {
        println!("Event: {}", event);
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

    fn get_module(&self, bytecode: &[u8], limit: u64) -> Result<RuntimeModule> {
        let as_module = ASModule::new(bytecode, limit, GasCosts::default(), Compiler::SP)?;
        let module = RuntimeModule::ASModule(as_module);
        Ok(module)
    }

    fn get_owned_addresses(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn ds_entry_exists(&self, _address: &str, _key: &[u8]) -> Result<bool> {
        Ok(false)
    }

    fn hash(&self, _data: &[u8]) -> Result<[u8; 32]> {
        unimplemented!()
    }

    fn append_ds_value(&self, _address: &str, _key: &[u8], _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn delete_ds_entry(&self, _address: &str, _key: &[u8]) -> Result<()> {
        Ok(())
    }

    fn get_ds_value(&self, _address: &str, _key: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn set_ds_value(&self, _address: &str, _key: &[u8], value: &[u8]) -> Result<()> {
        let mut bytes = self.0.lock().clone();
        bytes.insert(String::from_str("print").unwrap(), value.to_vec());
        Ok(())
    }

    fn signature_verify(&self, _data: &[u8], _signature: &str, _public_key: &str) -> Result<bool> {
        Ok(false)
    }

    fn unsafe_random(&self) -> Result<i64> {
        Ok(0)
    }

    fn get_balance(&self, _address: &str) -> Result<u64> {
        Ok(1)
    }

    fn set_bytecode(&self, address: &str, bytecode: &[u8]) -> Result<()> {
        self.0.lock().insert(address.to_string(), bytecode.to_vec());
        Ok(())
    }

    fn get_call_coins(&self) -> Result<u64> {
        Ok(0)
    }

    fn create_module(&self, module: &[u8]) -> Result<String> {
        let address = String::from("get_string");
        self.0.lock().insert(address.clone(), module.to_vec());
        Ok(address)
    }

    fn send_message(
        &self,
        _target_address: &str,
        _target_handler: &str,
        _validity_start: (u64, u8),
        _validity_end: (u64, u8),
        _max_gas: u64,
        _raw_fee: u64,
        _coins: u64,
        _data: &[u8],
        _filter: Option<(&str, Option<&[u8]>)>,
    ) -> Result<()> {
        Ok(())
    }

    fn get_op_keys(&self, _prefix: &[u8]) -> Result<BTreeSet<Vec<u8>>> {
        Ok(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 11],
            vec![127, 128],
            vec![254, 255],
        ]
        .into_iter()
        .collect())
    }

    fn op_entry_exists(&self, key: &[u8]) -> Result<bool> {
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        Ok(ds.contains_key(key))
    }

    fn get_op_value(&self, key: &[u8]) -> Result<Vec<u8>> {
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
        // Using chrono as a test dummy implementation to make sure the ABI is called correctly
        // Note that Massa node implementation uses the time of the context slot
        // in order to ensure determinism, not the UTC time
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
}

#[cfg(feature = "gas_calibration")]
pub mod tests_gas_calibration;
#[cfg(not(feature = "gas_calibration"))]
pub mod tests_runtime;

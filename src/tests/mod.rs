use crate::types::{Interface, InterfaceClone};

use anyhow::{bail, Result};
use parking_lot::Mutex;

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

    fn get_balance(&self) -> Result<u64> {
        Ok(1)
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

    fn address_from_public_key(&self, public_key: &str) -> Result<String> {
        Ok(public_key.to_string())
    }

    fn exit_success(&self) -> Result<()> {
        Ok(())
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

    fn get_module(&self, _address: &str) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn get_owned_addresses(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn has_data(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }

    fn has_data_for(&self, _address: &str, _key: &str) -> Result<bool> {
        Ok(false)
    }

    fn hash(&self, data: &[u8]) -> Result<String> {
        Ok(String::from_utf8(data.to_vec())?)
    }

    fn module_called(&self) -> Result<()> {
        Ok(())
    }

    fn raw_append_data(&self, _key: &str, _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_append_data_for(&self, _address: &str, _key: &str, _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn raw_delete_data(&self, _key: &str) -> Result<()> {
        Ok(())
    }

    fn raw_delete_data_for(&self, _address: &str, _key: &str) -> Result<()> {
        Ok(())
    }

    fn raw_get_data_for(&self, _address: &str, _key: &str) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn raw_set_data(&self, _key: &str, value: &[u8]) -> Result<()> {
        let mut bytes = self.0.lock().clone();
        bytes.insert(String::from_str("print").unwrap(), value.to_vec());
        Ok(())
    }

    fn raw_set_data_for(&self, _address: &str, _key: &str, value: &[u8]) -> Result<()> {
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

    fn get_balance_for(&self, _address: &str) -> Result<u64> {
        Ok(1)
    }

    fn raw_set_bytecode_for(&self, address: &str, bytecode: &[u8]) -> Result<()> {
        self.0.lock().insert(address.to_string(), bytecode.to_vec());
        Ok(())
    }

    fn raw_set_bytecode(&self, bytecode: &[u8]) -> Result<()> {
        let address = String::from("get_string");
        self.0.lock().insert(address, bytecode.to_vec());
        Ok(())
    }

    fn print(&self, message: &str) -> Result<()> {
        println!("{}", message);
        self.0
            .lock()
            .insert("print".into(), message.as_bytes().to_vec());
        Ok(())
    }

    fn raw_get_data(&self, _: &str) -> Result<Vec<u8>> {
        let bytes = self.0.lock().clone();
        match bytes.get(&"print".to_string()) {
            Some(bytes) => Ok(bytes.clone()),
            _ => Ok(vec![]),
        }
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
        _gas_price: u64,
        _coins: u64,
        _data: &[u8],
    ) -> Result<()> {
        Ok(())
    }

    fn get_op_keys(&self) -> Result<Vec<Vec<u8>>> {
        Ok(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 11],
            vec![127, 128],
            vec![254, 255],
        ])
    }

    fn has_op_key(&self, key: &[u8]) -> Result<bool> {
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
        Ok(0)
    }
}

#[cfg(feature = "gas_calibration")]
pub mod tests_gas_calibration;
#[cfg(not(feature = "gas_calibration"))]
pub mod tests_runtime;

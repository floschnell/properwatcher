use crate::models::ContractType;
use crate::models::PropertyType;

#[derive(Clone, Debug)]
pub struct Config {
  pub address: String,
  pub city: String,
  pub crawler: String,
  pub property_type: PropertyType,
  pub contract_type: ContractType,
}

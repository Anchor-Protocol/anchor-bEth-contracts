mod tax_querier;

pub use tax_querier::deduct_tax;
pub mod reward;

#[cfg(test)]
mod mock_querier;

#[cfg(test)]
mod testing;

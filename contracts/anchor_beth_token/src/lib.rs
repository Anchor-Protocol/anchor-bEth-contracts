pub mod contract;
pub mod msg;
pub mod state;

mod handler;

#[cfg(test)]
mod testing;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);

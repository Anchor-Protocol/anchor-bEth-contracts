use cosmwasm_std::{StdResult, Uint128};

pub(crate) fn convert_to_wormhole_decimals(
    amount: Uint128,
    anchor_decimals: u8,
    wormhole_decimals: u8,
) -> StdResult<Uint128> {
    if anchor_decimals > wormhole_decimals {
        let decimal_fraction =
            Uint128::new(10u128).saturating_pow((anchor_decimals - wormhole_decimals) as u32);
        Ok(amount.checked_div(decimal_fraction).unwrap())
    } else {
        let decimal_fraction =
            Uint128::new(10u128).saturating_pow((wormhole_decimals - anchor_decimals) as u32);
        Ok(amount.checked_mul(decimal_fraction).unwrap())
    }
}

pub(crate) fn convert_to_anchor_decimals(
    amount: Uint128,
    anchor_decimals: u8,
    wormhole_decimals: u8,
) -> StdResult<Uint128> {
    if anchor_decimals > wormhole_decimals {
        let decimal_fraction =
            Uint128::new(10u128).saturating_pow((anchor_decimals - wormhole_decimals) as u32);
        Ok(amount.checked_mul(decimal_fraction).unwrap())
    } else {
        let decimal_fraction =
            Uint128::new(10u128).saturating_pow((wormhole_decimals - anchor_decimals) as u32);
        Ok(amount.checked_div(decimal_fraction).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_wormhole_decimals() {
        let a = Uint128::new(100000000);
        let b = 4;
        let c = 6;
        let d = convert_to_wormhole_decimals(a, b, c).unwrap();
        assert_eq!(d, Uint128::new(10000000000));
    }

    #[test]
    fn test_convert_to_anchor_decimals() {
        let a = Uint128::new(100000000);
        let b = 4;
        let c = 6;
        let d = convert_to_anchor_decimals(a, b, c).unwrap();
        assert_eq!(d, Uint128::new(1000000));
    }
}

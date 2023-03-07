use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Api, CanonicalAddr, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use haloswap::asset::{AssetInfoRaw, PairInfo, PairInfoRaw};

#[cw_serde]
pub struct Config {
    pub owner: CanonicalAddr,
    pub pair_code_id: u64,
    pub token_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct TmpPairInfo {
    pub pair_key: Vec<u8>,
    pub asset_infos: [AssetInfoRaw; 2],
    pub asset_decimals: [u8; 2],
}

pub const TMP_PAIR_INFO: Item<TmpPairInfo> = Item::new("tmp_pair_info");
pub const PAIRS: Map<&[u8], PairInfoRaw> = Map::new("pair_info");

pub fn pair_key(asset_infos: &[AssetInfoRaw; 2]) -> Vec<u8> {
    let mut asset_infos = asset_infos.to_vec();
    asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    [asset_infos[0].as_bytes(), asset_infos[1].as_bytes()].concat()
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_pairs(
    storage: &dyn Storage,
    api: &dyn Api,
    start_after: Option<[AssetInfoRaw; 2]>,
    limit: Option<u32>,
) -> StdResult<Vec<PairInfo>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after).map(Bound::ExclusiveRaw);

    PAIRS
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            v.to_normal(api)
        })
        .collect::<StdResult<Vec<PairInfo>>>()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<[AssetInfoRaw; 2]>) -> Option<Vec<u8>> {
    start_after.map(|asset_infos| {
        let mut asset_infos = asset_infos.to_vec();
        asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut v = [asset_infos[0].as_bytes(), asset_infos[1].as_bytes()]
            .concat()
            .as_slice()
            .to_vec();
        v.push(1);
        v
    })
}

// key : asset info / value: decimals
pub const ALLOW_NATIVE_TOKENS: Map<&[u8], u8> = Map::new("allow_native_token");
pub fn add_allow_native_token(
    storage: &mut dyn Storage,
    denom: String,
    decimals: u8,
) -> StdResult<()> {
    ALLOW_NATIVE_TOKENS.save(storage, denom.as_bytes(), &decimals)
}
#[cfg(test)]
mod setting_pagination {
    use cosmwasm_std::Uint128;
    use haloswap::{asset::CreatePairRequirements, mock_querier::mock_dependencies};

    use super::*;

    #[test]
    fn test_read_pairs() {
        let mut deps = mock_dependencies(&[]);
        let api = deps.api;
        let asset_infos = [
            AssetInfoRaw::NativeToken {
                denom: "uaura".to_string(),
            },
            AssetInfoRaw::NativeToken {
                denom: "uatom".to_string(),
            },
        ];
        let pair_key1 = pair_key(&asset_infos);
        PAIRS
            .save(
                deps.as_mut().storage,
                pair_key1.as_slice(),
                &PairInfoRaw {
                    contract_addr: api.addr_canonicalize("pair1").unwrap(),
                    liquidity_token: api.addr_canonicalize("lp1").unwrap(),
                    asset_infos,
                    asset_decimals: [6u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                },
            )
            .unwrap();

        let asset_infos = [
            AssetInfoRaw::NativeToken {
                denom: "uatom".to_string(),
            },
            AssetInfoRaw::NativeToken {
                denom: "uusd".to_string(),
            },
        ];

        let pair_key2 = pair_key(&asset_infos);
        PAIRS
            .save(
                deps.as_mut().storage,
                pair_key2.as_slice(),
                &PairInfoRaw {
                    contract_addr: api.addr_canonicalize("pair2").unwrap(),
                    liquidity_token: api.addr_canonicalize("lp2").unwrap(),
                    asset_infos,
                    asset_decimals: [6u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                },
            )
            .unwrap();

        let asset_infos = [
            AssetInfoRaw::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfoRaw::NativeToken {
                denom: "uaura".to_string(),
            },
        ];
        let pair_key3 = pair_key(&asset_infos);
        PAIRS
            .save(
                deps.as_mut().storage,
                pair_key3.as_slice(),
                &PairInfoRaw {
                    contract_addr: api.addr_canonicalize("pair3").unwrap(),
                    liquidity_token: api.addr_canonicalize("lp3").unwrap(),
                    asset_infos,
                    asset_decimals: [6u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                },
            )
            .unwrap();

        let pairs = read_pairs(deps.as_ref().storage, deps.as_ref().api, None, None).unwrap();
        println!("PAIR: {:?}", pairs);
        assert_eq!(pairs.len(), 3);

        let pairs = read_pairs(
            deps.as_ref().storage,
            deps.as_ref().api,
            Some([
                AssetInfoRaw::NativeToken {
                    denom: "uaura".to_string(),
                },
                AssetInfoRaw::NativeToken {
                    denom: "uatom".to_string(),
                },
            ]),
            None,
        )
        .unwrap();
        assert_eq!(pairs.len(), 2);

        let pairs = read_pairs(
            deps.as_ref().storage,
            deps.as_ref().api,
            Some([
                AssetInfoRaw::NativeToken {
                    denom: "uaura".to_string(),
                },
                AssetInfoRaw::NativeToken {
                    denom: "uatom".to_string(),
                },
            ]),
            Some(1),
        )
        .unwrap();
        assert_eq!(pairs.len(), 1);
    }
}

#[cfg(test)]
mod allow_native_token {

    use haloswap::mock_querier::mock_dependencies;

    use super::*;

    #[test]
    fn normal() {
        let mut deps = mock_dependencies(&[]);
        let denom = "uluna".to_string();
        let decimals = 6u8;

        add_allow_native_token(deps.as_mut().storage, denom.to_string(), decimals).unwrap();

        assert_eq!(
            decimals,
            ALLOW_NATIVE_TOKENS
                .load(deps.as_ref().storage, denom.as_bytes())
                .unwrap()
        )
    }

    #[test]
    fn duplicate_register_will_append() {
        let mut deps = mock_dependencies(&[]);
        let denom = "uluna".to_string();

        add_allow_native_token(deps.as_mut().storage, denom.to_string(), 6u8).unwrap();

        assert_eq!(
            ALLOW_NATIVE_TOKENS
                .load(deps.as_ref().storage, denom.as_bytes())
                .unwrap(),
            6u8
        );

        add_allow_native_token(deps.as_mut().storage, denom.to_string(), 7u8).unwrap();
        assert_eq!(
            ALLOW_NATIVE_TOKENS
                .load(deps.as_ref().storage, denom.as_bytes())
                .unwrap(),
            7u8
        );
    }
}

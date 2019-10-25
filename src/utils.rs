use serde::Serializer;
use chrono::{DateTime, Utc};
use chrono::offset::TimeZone;

use nimiq_hash::Blake2bHash;
use nimiq_primitives::coin::Coin;
use nimiq_primitives::account::AccountType;
use nimiq_keys::Address;


pub(crate) fn serialize_blake2b<S>(hash: &Blake2bHash, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    serializer.serialize_str(&hash.to_hex())
}

pub(crate) fn serialize_with_hex<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    serializer.serialize_str(&hex::encode(data))
}

pub(crate) fn serialize_with_beserial<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: beserial::Serialize,
          S: Serializer,
{
    let mut buf = Vec::with_capacity(x.serialized_size());
    x.serialize(&mut buf).unwrap(); // TODO: handle error
    serialize_with_hex(&buf, serializer)
}

pub(crate) fn serialize_with_beserial_opt<T, S>(x_opt: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where T: beserial::Serialize,
          S: Serializer,
{
    match &x_opt {
        Some(x) => serialize_with_beserial(x, serializer),
        None => serializer.serialize_none(),
    }
}

pub(crate) fn serialize_coin_formatted<S>(coin: &Coin, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    serializer.serialize_str(&format!("{}", coin))
}

pub(crate) fn serialize_special_account_type<S>(account_type: &AccountType, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    if *account_type == AccountType::Basic {
        serializer.serialize_none()
    }
    else {
        let s = match account_type {
            AccountType::Basic => unreachable!(),
            AccountType::Vesting => "vesting",
            AccountType::HTLC => "htlc",
            AccountType::Staking => "staking",
        };
        serializer.serialize_str(s)
    }
}

pub(crate) fn serialize_address<S>(address: &Address, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    serializer.serialize_str(&address.to_user_friendly_address())
}

pub(crate) fn null_hash_opt(hash: Blake2bHash) -> Option<Blake2bHash> {
    if hash.as_bytes().iter().all(|b| *b == 0) {
        None
    }
    else {
        Some(hash)
    }
}

pub fn nimiq_to_chrono(timestamp: u64) -> DateTime<Utc> {
    Utc.timestamp((timestamp / 1000) as i64, ((timestamp % 1000) * 1000) as u32)
}

pub fn short_hash(hash: &Blake2bHash) -> String {
    let mut s = hash.to_hex();
    s.truncate(8);
    s
}
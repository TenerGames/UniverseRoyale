use std::io::{Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::systems::stream_connection::{BytesOptions, OrderOptions, TcpSettings};
use paste::paste;

macro_rules! define_read_write_endian {
    ($name:ident, $ty:ty, $read:ident, $read_le:ident, $write:ident, $write_le:ident) => {
        paste! {
            pub async fn [<$name _read>](read_half: &mut OwnedReadHalf, order: &OrderOptions) -> Result<$ty, Error> {
                match order {
                    OrderOptions::LittleEndian => read_half.$read_le().await,
                    OrderOptions::BigEndian    => read_half.$read().await,
                }
            }

            pub async fn [<$name _write>](write_half: &mut OwnedWriteHalf, value: $ty, order: &OrderOptions) -> Result<(), Error> {
                match order {
                    OrderOptions::LittleEndian => write_half.$write_le(value).await.map(|_| ()),
                    OrderOptions::BigEndian    => write_half.$write(value).await.map(|_| ()),
                }
            }
        }
    };
}

macro_rules! define_read_write_basic {
    ($name:ident, $ty:ty, $read:ident, $write:ident) => {
        paste! {
            pub async fn [<$name _read>](read_half: &mut OwnedReadHalf, _order: &OrderOptions) -> Result<$ty, Error> {
                read_half.$read().await
            }

            pub async fn [<$name _write>](write_half: &mut OwnedWriteHalf, value: $ty, _order: &OrderOptions) -> Result<(), Error> {
                write_half.$write(value).await.map(|_| ())
            }
        }
    };
}

define_read_write_endian!(u16, u16, read_u16, read_u16_le, write_u16, write_u16_le);
define_read_write_endian!(u32, u32, read_u32, read_u32_le, write_u32, write_u32_le);
define_read_write_endian!(u64, u64, read_u64, read_u64_le, write_u64, write_u64_le);
define_read_write_endian!(u128, u128, read_u128, read_u128_le, write_u128, write_u128_le);

define_read_write_endian!(i16, i16, read_i16, read_i16_le, write_i16, write_i16_le);
define_read_write_endian!(i32, i32, read_i32, read_i32_le, write_i32, write_i32_le);
define_read_write_endian!(i64, i64, read_i64, read_i64_le, write_i64, write_i64_le);
define_read_write_endian!(i128, i128, read_i128, read_i128_le, write_i128, write_i128_le);

define_read_write_endian!(f32, f32, read_f32, read_f32_le, write_f32, write_f32_le);
define_read_write_endian!(f64, f64, read_f64, read_f64_le, write_f64, write_f64_le);

define_read_write_basic!(u8, u8, read_u8, write_u8);
define_read_write_basic!(i8, i8, read_i8, write_i8);

pub async fn read_value_from_settings(
    read_half: &mut OwnedReadHalf,
    tcp_settings: &TcpSettings,
) -> Result<Vec<u8>, Error> {
    let order = &tcp_settings.order;

    match tcp_settings.bytes {
        // unsigned
        BytesOptions::U8 => Ok(vec![0u8; u8_read(read_half, order).await? as usize]),
        BytesOptions::U16 => Ok(vec![0u8; u16_read(read_half, order).await? as usize]),
        BytesOptions::U32 => Ok(vec![0u8; u32_read(read_half, order).await? as usize]),
        BytesOptions::U64 => Ok(vec![0u8; u64_read(read_half, order).await? as usize]),
        BytesOptions::U128 => Ok(vec![0u8; u128_read(read_half, order).await? as usize]),

        // signed
        BytesOptions::I8 => Ok(vec![0u8; i8_read(read_half, order).await? as usize]),
        BytesOptions::I16 => Ok(vec![0u8; i16_read(read_half, order).await? as usize]),
        BytesOptions::I32 => Ok(vec![0u8; i32_read(read_half, order).await? as usize]),
        BytesOptions::I64 => Ok(vec![0u8; i64_read(read_half, order).await? as usize]),
        BytesOptions::I128 => Ok(vec![0u8; i128_read(read_half, order).await? as usize]),

        // floats
        BytesOptions::F32 => Ok(vec![0u8; f32_read(read_half, order).await? as usize]),
        BytesOptions::F64 => Ok(vec![0u8; f64_read(read_half, order).await? as usize]),
    }
}

pub async fn write_value_from_settings(
    write_half: &mut OwnedWriteHalf,
    encoded: &Vec<u8>,
    tcp_settings: &TcpSettings,
) -> Result<(), Error> {
    let order = &tcp_settings.order;

    match tcp_settings.bytes {
        // unsigned
        BytesOptions::U8 => u8_write(write_half, encoded.len() as u8, order).await,
        BytesOptions::U16 => u16_write(write_half, encoded.len() as u16, order).await,
        BytesOptions::U32 => u32_write(write_half, encoded.len() as u32, order).await,
        BytesOptions::U64 => u64_write(write_half, encoded.len() as u64, order).await,
        BytesOptions::U128 => u128_write(write_half, encoded.len() as u128, order).await,

        // signed
        BytesOptions::I8 => i8_write(write_half, encoded.len() as i8, order).await,
        BytesOptions::I16 => i16_write(write_half, encoded.len() as i16, order).await,
        BytesOptions::I32 => i32_write(write_half, encoded.len() as i32, order).await,
        BytesOptions::I64 => i64_write(write_half, encoded.len() as i64, order).await,
        BytesOptions::I128 => i128_write(write_half, encoded.len() as i128, order).await,

        // floats
        BytesOptions::F32 => f32_write(write_half, encoded.len() as f32, order).await,
        BytesOptions::F64 => f64_write(write_half, encoded.len() as f64, order).await,
    }
}
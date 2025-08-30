use std::io::{Error, ErrorKind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::systems::stream_connection::{BytesOptions, OrderOptions, TcpSettings};

pub async fn read_u32(read_half: &mut OwnedReadHalf, order: &OrderOptions) -> Result<u32, Error> {
    if order == &OrderOptions::LittleEndian{
        match read_half.read_u32_le().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match read_half.read_u32().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn read_f32(read_half: &mut OwnedReadHalf, order: &OrderOptions) -> Result<f32, Error> {
    if order == &OrderOptions::LittleEndian{
        match read_half.read_f32_le().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match read_half.read_f32().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn read_u64(read_half: &mut OwnedReadHalf, order: &OrderOptions) -> Result<u64, Error> {
    if order == &OrderOptions::LittleEndian{
        match read_half.read_u64_le().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match read_half.read_u64().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn read_f64(read_half: &mut OwnedReadHalf, order: &OrderOptions) -> Result<f64, Error> {
    if order == &OrderOptions::LittleEndian{
        match read_half.read_f64_le().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match read_half.read_f64().await {
            Ok(length) => {
                Ok(length)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn write_u32(write_half: &mut OwnedWriteHalf, length: u32, order: &OrderOptions) -> Result<(), Error> {
    if order == &OrderOptions::LittleEndian{
        match write_half.write_u32_le(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match write_half.write_u32(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn write_f32(write_half: &mut OwnedWriteHalf, length: f32, order: &OrderOptions) -> Result<(), Error> {
    if order == &OrderOptions::LittleEndian{
        match write_half.write_f32_le(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match write_half.write_f32(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn write_u64(write_half: &mut OwnedWriteHalf, length: u64, order: &OrderOptions) -> Result<(), Error> {
    if order == &OrderOptions::LittleEndian{
        match write_half.write_u64_le(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match write_half.write_u64(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn write_f64(write_half: &mut OwnedWriteHalf, length: f64, order: &OrderOptions) -> Result<(), Error> {
    if order == &OrderOptions::LittleEndian{
        match write_half.write_f64_le(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        match write_half.write_f64(length).await {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub async fn read_value_from_settings(read_half: &mut OwnedReadHalf, tcp_settings: &TcpSettings) -> Result<Vec<u8>, Error>{
    if tcp_settings.bytes == BytesOptions::U32 {
        match read_u32(read_half, &tcp_settings.order).await{
            Ok(length) => {
                Ok(vec![0u8; length as usize])
            },
            Err(e) => {
                Err(e)
            }
        }
    }else if tcp_settings.bytes == BytesOptions::F32 {
        match read_f32(read_half, &tcp_settings.order).await{
            Ok(length) => {
                Ok(vec![0u8; length as usize])
            },
            Err(e) => {
                Err(e)
            }
        }
    }else if tcp_settings.bytes == BytesOptions::U64 {
        match read_u64(read_half, &tcp_settings.order).await{
            Ok(length) => {
                Ok(vec![0u8; length as usize])
            },
            Err(e) => {
                Err(e)
            }
        }
    }else if tcp_settings.bytes == BytesOptions::F64 {
        match read_f64(read_half, &tcp_settings.order).await{
            Ok(length) => {
                Ok(vec![0u8; length as usize])
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        Err(Error::new(ErrorKind::Other,"Not implemented"))
    }
}

pub async fn write_value_from_settings(write_half: &mut OwnedWriteHalf, encoded: &Vec<u8>, tcp_settings: &TcpSettings) -> Result<(), Error>{
    if tcp_settings.bytes == BytesOptions::U32 {
        match write_u32(write_half, encoded.len() as u32, &tcp_settings.order).await{
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else if tcp_settings.bytes == BytesOptions::F32 {
        match write_f32(write_half, encoded.len() as f32, &tcp_settings.order).await{
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else if tcp_settings.bytes == BytesOptions::U64 {
        match write_u64(write_half, encoded.len() as u64, &tcp_settings.order).await{
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else if tcp_settings.bytes == BytesOptions::F64 {
        match write_f64(write_half, encoded.len() as f64, &tcp_settings.order).await{
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }else {
        Err(Error::new(ErrorKind::Other,"Not implemented"))
    }
}
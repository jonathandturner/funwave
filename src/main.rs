use std::io;
use std::io::prelude::*;
use std::fs::File;

mod wave_format {
    pub const PCM        : u16 = 0x0001;
    pub const IEEE_FLOAT : u16 = 0x0003;
    pub const A_LAW      : u16 = 0x0006;
    pub const MU_LAW     : u16 = 0x0007;
    pub const EXTENSIBLE : u16 = 0xFFFE;
}

#[derive(Debug)]
struct FmtChunk {
    tag: u16,
    num_chans: u16,
    samples_per_sec: u32,
    avg_bytes_per_sec: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn load_4byte_id(file: &mut File, id:&[u8; 4]) -> Result<(), io::Error> {
    use std::io::{Error, ErrorKind};

    let mut buffer = [0; 4];
    try!(file.read(&mut buffer));
    
    if &buffer != id {
        return Err(Error::new(ErrorKind::InvalidInput, "File not in WAVE format"));
    }
    
    Ok(())
}

fn load_u32_le(file: &mut File) -> Result<u32, io::Error> {
    let mut buffer = [0; 4];

    try!(file.read(&mut buffer));
    
    Ok(buffer[0] as u32 + ((buffer[1] as u32) << 8) +
        ((buffer[2] as u32) << 16) + ((buffer[3] as u32) << 24))    
}

fn load_u16_le(file: &mut File) -> Result<u16, io::Error> {
    let mut buffer = [0; 2];

    try!(file.read(&mut buffer));
    
    Ok(buffer[0] as u16 + ((buffer[1] as u16) << 8))    
}

fn load_wave_format(file: &mut File) -> Result<u16, io::Error> {
    use std::io::{Error, ErrorKind};

    let fmt_tag = try!(load_u16_le(file)); 

    match fmt_tag {
        wave_format::PCM        | 
        wave_format::IEEE_FLOAT | 
        wave_format::A_LAW      | 
        wave_format::MU_LAW     | 
        wave_format::EXTENSIBLE => Ok(fmt_tag),
        _  => Err(Error::new(ErrorKind::InvalidInput, "File not in WAVE format"))
    }    
}

fn load_wave(filename: &String) -> Result<(), io::Error> {
    let mut f = try!(File::open(filename));

    //Read the RIFF header
    try!(load_4byte_id(&mut f, b"RIFF"));
    let riff_size = try!(load_u32_le(&mut f)) - 4;
    try!(load_4byte_id(&mut f, b"WAVE"));
                
    //Next, the WAVE chunk
    //which starts with a fmt chunk
    try!(load_4byte_id(&mut f, b"fmt "));
    let fmt_chunk_size = try!(load_u32_le(&mut f));
    
    let result = FmtChunk {
        tag: try!(load_wave_format(&mut f)),
        num_chans: try!(load_u16_le(&mut f)),
        samples_per_sec: try!(load_u32_le(&mut f)),
        avg_bytes_per_sec: try!(load_u32_le(&mut f)),
        block_align: try!(load_u16_le(&mut f)),
        bits_per_sample: try!(load_u16_le(&mut f))
    };
    
    println!("Riff size: {}", riff_size);
    println!("fmt size: {}", fmt_chunk_size);
    println!("fmt: {:?}", result);

    Ok(())
}

fn main() {
    use std::env;
    
    let args = env::args();
    
    for arg in args.skip(1) {
        println!("Result for {} is {:?}", &arg, load_wave(&arg));
    }
}

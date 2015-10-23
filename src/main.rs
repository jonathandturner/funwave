use std::io;
use std::io::prelude::*;
use std::fs::File;

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

fn load_wave(filename: &String) -> Result<(), io::Error> {
    let mut f = try!(File::open(filename));

    try!(load_4byte_id(&mut f, b"RIFF"));
    
    let riff_size = try!(load_u32_le(&mut f)) - 4;
    println!("Riff size: {}", riff_size);

    try!(load_4byte_id(&mut f, b"WAVE"));
    
    //Next, the WAVES chunk
    //which starts with a fmt chunk
    try!(load_4byte_id(&mut f, b"fmt "));
     
    Ok(())
}

fn main() {
    use std::env;
    
    let args = env::args();
    
    for arg in args.skip(1) {
        println!("Result for {} is {:?}", &arg, load_wave(&arg));
    }
}

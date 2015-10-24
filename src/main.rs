use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fmt; //for custom Debug

mod wave_format {
    pub const PCM        : u16 = 0x0001;
    pub const IEEE_FLOAT : u16 = 0x0003;
    pub const A_LAW      : u16 = 0x0006;
    pub const MU_LAW     : u16 = 0x0007;
    pub const EXTENSIBLE : u16 = 0xFFFE;
}

enum WaveData {
    BytePerSample(Vec<Vec<i8>>),
    WordPerSample(Vec<Vec<u16>>)
}
impl fmt::Debug for WaveData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result;
        match *self {
            WaveData::BytePerSample(ref bytes) => {
                result = write!(f, "(byte)");
                for chan in bytes.iter().take(1) {
                    for byte in chan.iter().take(5) {
                        result = write!(f, "0x{0:x} ", byte);
                    }
                }
            }
            WaveData::WordPerSample(ref words) => {
                result = write!(f, "(word)");
                for chan in words.iter().take(1) {
                    for word in chan.iter().take(5) {
                        result = write!(f, "0x{0:x} ", word);
                    }
                }
            }
        }
        result
    }
}
  
#[derive(Debug)]
struct Wave {
    tag: u16,
    num_chans: u16,
    samples_per_sec: u32,
    avg_bytes_per_sec: u32,
    block_align: u16,
    samples: WaveData
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

fn load_wave_samples(file: &mut File, num_chans: u16, bits_per_sample: u16) -> Result<WaveData, io::Error> {
    use std::io::{Error, ErrorKind};

    let mut raw_data = Vec::new();
    try!(file.read_to_end(&mut raw_data));

    match bits_per_sample {
        8 => {
            let mut output : Vec<Vec<i8>> = Vec::new();
            
            for _ in 0..num_chans {
                output.push(Vec::new());
            }
            
            let mut chan_cursor: usize = 0;
            
            for byte in raw_data {
                output[chan_cursor].push(byte as i8);
                chan_cursor = (chan_cursor + 1) % (num_chans as usize);
            }
            
            if chan_cursor == 0 {
                Ok(WaveData::BytePerSample(output))
            }
            else {
                Err(Error::new(ErrorKind::InvalidInput, "Incomplete WAVE data"))
            }
        },
        16 => {
            let mut output : Vec<Vec<u16>> = Vec::new();
            
            for _ in 0..num_chans {
                output.push(Vec::new());
            }
            
            let mut chan_cursor: usize = 0;
            let mut word_cursor = 0;
            let mut word_cache = 0;
            
            for byte in raw_data {
                word_cache = word_cache + ((byte as u16) << word_cursor);
                if word_cursor == 8 {
                    output[chan_cursor].push(word_cache);
                    chan_cursor = (chan_cursor + 1) % (num_chans as usize);
                    word_cache = 0;
                    word_cursor = 0;
                }
                else {
                    word_cursor = 8;
                }
            }
            
            if chan_cursor == 0 && word_cursor == 0 {
                Ok(WaveData::WordPerSample(output))
            }
            else {
                Err(Error::new(ErrorKind::InvalidInput, "Incomplete WAVE data"))
            }        
        },
        _  => Err(Error::new(ErrorKind::InvalidInput, "File not in WAVE format"))
    }
}

fn load_wave(filename: &String) -> Result<Wave, io::Error> {
    let mut f = try!(File::open(filename));

    //Read the RIFF header
    try!(load_4byte_id(&mut f, b"RIFF"));
    try!(load_u32_le(&mut f));  // riff_size 
    try!(load_4byte_id(&mut f, b"WAVE"));
                
    //Next, the WAVE chunk
    //which starts with a fmt chunk
    try!(load_4byte_id(&mut f, b"fmt "));
    try!(load_u32_le(&mut f)); // fmt_chunk_size 

    let tag = try!(load_wave_format(&mut f));
    let num_chans = try!(load_u16_le(&mut f));
    let samples_per_sec = try!(load_u32_le(&mut f));
    let avg_bytes_per_sec = try!(load_u32_le(&mut f));
    let block_align = try!(load_u16_le(&mut f));
    let bits_per_sample = try!(load_u16_le(&mut f));
    
    //Finally, load the samples for the wave
    try!(load_4byte_id(&mut f, b"data"));
    let samples = try!(load_wave_samples(&mut f, num_chans, bits_per_sample));
    
    Ok(Wave {
        tag: tag,
        num_chans: num_chans,
        samples_per_sec: samples_per_sec,
        avg_bytes_per_sec: avg_bytes_per_sec,
        block_align: block_align,
        samples: samples
    })
}

fn main() {
    use std::env;
    
    let args = env::args();
    
    for arg in args.skip(1) {
        println!("Result for {} is {:?}", &arg, load_wave(&arg));
    }
}

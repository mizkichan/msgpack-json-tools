#![feature(int_to_from_bytes, try_from, box_syntax)]
extern crate structopt;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt()]
struct Opt {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = opt
        .input
        .map(|path| File::open(path).unwrap())
        .ok_or_else(|| stdin.lock());
    let mut output = opt
        .output
        .map(|path| File::create(path).unwrap())
        .ok_or_else(|| stdout.lock());

    let mut input = match input {
        Ok(ref mut file) => BufReader::<&mut Read>::new(file),
        Err(ref mut stdin) => BufReader::<&mut Read>::new(stdin),
    };
    let mut output = match output {
        Ok(ref mut file) => BufWriter::<&mut Write>::new(file),
        Err(ref mut stdout) => BufWriter::<&mut Write>::new(stdout),
    };

    main_impl(&mut input, &mut output);
}

fn main_impl(stdin: &mut impl Read, stdout: &mut impl Write) {
    let byte = {
        let mut buf = [0u8; 1];
        stdin.read_exact(&mut buf).unwrap();
        buf[0]
    };

    match byte {
        // positive fixint
        0x00..=0x7f => {
            write!(stdout, "{}", byte as u8);
        }

        // fixmap
        0x80..=0x8f => {
            let length = (byte & 0b0000_1111).into();
            print_map(length, stdin, stdout);
        }

        // fixarray
        0x90..=0x9f => {
            let length = (byte & 0b0000_1111).into();
            print_array(length, stdin, stdout);
        }

        // fixstr
        0xa0..=0xbf => {
            let length = (byte & 0b0001_1111).into();
            print_str(length, stdin, stdout);
        }

        // nil
        0xc0 => {
            assert_eq!(stdout.write(b"null").unwrap(), 4);
        }

        // never used
        0xc1 => panic!(),

        // false
        0xc2 => {
            assert_eq!(stdout.write(b"false").unwrap(), 5);
        }

        // true
        0xc3 => {
            assert_eq!(stdout.write(b"true").unwrap(), 4);
        }

        // bin 8
        0xc4 => unimplemented!(),

        // bin 16
        0xc5 => unimplemented!(),

        // bin 32
        0xc6 => unimplemented!(),

        // ext 8
        0xc7 => unimplemented!(),

        // ext 16
        0xc8 => unimplemented!(),

        // ext 32
        0xc9 => unimplemented!(),

        // float 32
        0xca => {
            write!(stdout, "{}", get_f32(stdin));
        }

        // float 64
        0xcb => {
            write!(stdout, "{}", get_f64(stdin));
        }

        // uint 8
        0xcc => {
            write!(stdout, "{}", get_u8(stdin));
        }

        // uint 16
        0xcd => {
            write!(stdout, "{}", get_u16(stdin));
        }

        // uint 32
        0xce => {
            write!(stdout, "{}", get_u32(stdin));
        }

        // uint 64
        0xcf => {
            write!(stdout, "{}", get_u64(stdin));
        }

        // int 8
        0xd0 => {
            write!(stdout, "{}", get_i8(stdin));
        }

        // int 16
        0xd1 => {
            write!(stdout, "{}", get_i16(stdin));
        }

        // int 32
        0xd2 => {
            write!(stdout, "{}", get_i32(stdin));
        }

        // int 64
        0xd3 => {
            write!(stdout, "{}", get_i64(stdin));
        }

        // fixext 1
        0xd4 => unimplemented!(),

        // fixext 2
        0xd5 => unimplemented!(),

        // fixext 4
        0xd6 => unimplemented!(),

        // fixext 8
        0xd7 => unimplemented!(),

        // fixext 16
        0xd8 => unimplemented!(),

        // str 8
        0xd9 => {
            let length = get_u8(stdin).into();
            print_str(length, stdin, stdout);
        }

        // str 16
        0xda => {
            let length = get_u16(stdin).into();
            print_str(length, stdin, stdout);
        }

        // str 32
        0xdb => {
            let length = get_u32(stdin).into();
            print_str(length, stdin, stdout);
        }

        // array 16
        0xdc => {
            let length = get_u16(stdin).into();
            print_array(length, stdin, stdout);
        }

        // array 32
        0xdd => {
            let length = get_u32(stdin).try_into().unwrap();
            print_array(length, stdin, stdout);
        }

        // map 16
        0xde => {
            let length = get_u16(stdin).into();
            print_map(length, stdin, stdout);
        }

        // map 32
        0xdf => {
            let length = get_u32(stdin).try_into().unwrap();
            print_map(length, stdin, stdout);
        }

        // negative fixint
        0xe0..=0xff => {
            write!(stdout, "{}", byte as i8);
        }

        _ => unreachable!(),
    }
}

fn get_1byte(stdin: &mut Read) -> [u8; 1] {
    let mut buf = [0; 1];
    stdin.read_exact(&mut buf).unwrap();
    buf
}

fn get_2bytes(stdin: &mut Read) -> [u8; 2] {
    let mut buf = [0; 2];
    stdin.read_exact(&mut buf).unwrap();
    buf
}

fn get_4bytes(stdin: &mut Read) -> [u8; 4] {
    let mut buf = [0; 4];
    stdin.read_exact(&mut buf).unwrap();
    buf
}

fn get_8bytes(stdin: &mut Read) -> [u8; 8] {
    let mut buf = [0; 8];
    stdin.read_exact(&mut buf).unwrap();
    buf
}

fn get_u8(stdin: &mut Read) -> u8 {
    u8::from_be_bytes(get_1byte(stdin))
}

fn get_u16(stdin: &mut Read) -> u16 {
    u16::from_be_bytes(get_2bytes(stdin))
}

fn get_u32(stdin: &mut Read) -> u32 {
    u32::from_be_bytes(get_4bytes(stdin))
}

fn get_u64(stdin: &mut Read) -> u64 {
    u64::from_be_bytes(get_8bytes(stdin))
}

fn get_i8(stdin: &mut Read) -> i8 {
    i8::from_be_bytes(get_1byte(stdin))
}

fn get_i16(stdin: &mut Read) -> i16 {
    i16::from_be_bytes(get_2bytes(stdin))
}

fn get_i32(stdin: &mut Read) -> i32 {
    i32::from_be_bytes(get_4bytes(stdin))
}

fn get_i64(stdin: &mut Read) -> i64 {
    i64::from_be_bytes(get_8bytes(stdin))
}

fn get_f32(stdin: &mut Read) -> f32 {
    f32::from_bits(get_u32(stdin))
}

fn get_f64(stdin: &mut Read) -> f64 {
    f64::from_bits(get_u64(stdin))
}

fn print_map(length: usize, stdin: &mut impl Read, stdout: &mut impl Write) {
    assert_eq!(stdout.write(b"{").unwrap(), 1);
    for i in 0..length {
        main_impl(stdin, stdout);
        assert_eq!(stdout.write(b":").unwrap(), 1);
        main_impl(stdin, stdout);
        if i < length - 1 {
            assert_eq!(stdout.write(b",").unwrap(), 1);
        }
    }
    assert_eq!(stdout.write(b"}").unwrap(), 1);
}

fn print_array(length: usize, stdin: &mut impl Read, stdout: &mut impl Write) {
    assert_eq!(stdout.write(b"[").unwrap(), 1);
    for i in 0..length {
        main_impl(stdin, stdout);
        if i < length - 1 {
            assert_eq!(stdout.write(b",").unwrap(), 1);
        }
    }
    assert_eq!(stdout.write(b"]").unwrap(), 1);
}

fn print_str(length: u64, stdin: &mut impl Read, stdout: &mut impl Write) {
    let mut buf = String::new();
    assert_eq!(
        length as usize,
        stdin.take(length).read_to_string(&mut buf).unwrap()
    );
    write!(stdout, r#""{}""#, buf.replace('"', r#"\""#));
}

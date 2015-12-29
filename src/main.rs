use std::env;
use std::fs::File;
use std::io;

use cpu::Cpu;
use screen::BasicScreen;

mod cpu;
mod screen;

fn read_from_file(path: &str, memory: &mut [u16]) -> io::Result<usize> {
    use std::io::Read;

    let mut f = try!(File::open(path));
    let mut buf = [0u8; 2];
    let mut i = 0;

    loop {
        let res = try!(f.read(&mut buf));
        if res != 2 {
            break;
        }

        memory[i] = (buf[0] as u16) | ((buf[1] as u16) << 8);
        i += 1;
    };

    Ok(i)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 1 {
        panic!("Pass in a binary file");
    }

    let mut memory: [u16; cpu::MEM_SIZE] = [0; cpu::MEM_SIZE];

    let file_path = args.last().expect("Need a valid filename");
    read_from_file(&file_path, &mut memory).ok().expect("Failure reading program file");

    let screen = Box::new(BasicScreen::new());
    let mut cpu = Cpu::new(screen);
    cpu.load_memory(&mut memory);

    /*
    if args.len() > 1 {
        let first_arg = args.first().expect("");
        if first_arg == "-d" {
            cpu.print_memory();
        } else {
            panic!("Unknown parameter {}", first_arg);
        }
    } else {
    */
        while cpu.enabled {
            cpu.execute();
        }
    //}
}

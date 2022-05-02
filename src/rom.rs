use std::fs;
use std::error::Error;

pub fn header_info(header : &[u8]) {
  for v in header {
    print!("{} ", v);
  }
  println!();
  println!("PRG-ROM size LSB: {}", header[4]);
  println!("CHR-ROM size LSB: {}", header[5]);
  println!("{:#010b}", header[6]);
  println!("{:#010b}", header[7]);
  println!("{:#010b}", header[8]);
  println!("{:#010b}", header[9]);
  println!("{:#010b}", header[10]);
  println!("{:#010b}", header[11]);
  println!("{:#010b}", header[12]);
  println!("{:#010b}", header[13]);
  println!("{:#010b}", header[14]);
  println!("{:#010b}", header[15]);
}

pub fn nes_rom_load(filename : &str) -> Result<Vec<u8>, Box<dyn Error>> {

  let file = fs::read(filename)?;
  if file[0] == 'N' as u8 && file[1] == 'E' as u8
      && file[2] == 'S' as u8 && file[3] == 0x1A as u8 {
    println!("total len: {}", file.len());
    header_info(&file[..16])
  }
  Ok(file)
}

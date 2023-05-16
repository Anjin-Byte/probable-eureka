use byteorder::{LittleEndian, WriteBytesExt, ByteOrder};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// TIFF header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TiffHeader {
    pub byte_order: u16,
    pub magic_number: u16,
    pub ifd_offset: u32,
}

// TIFF IFD (Image File Directory) entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IfdEntry {
    pub tag: u16,
    pub field_type: u16,
    pub count: u32,
    pub value_offset: u32,
}

pub fn write_tiff_header<W: WriteBytesExt>(writer: &mut W, ifd_offset: u32) -> std::io::Result<()> {
    writer.write_u16::<LittleEndian>(0x4949)?; // Little-endian byte order
    writer.write_u16::<LittleEndian>(42)?; // Magic number
    writer.write_u32::<LittleEndian>(ifd_offset)?;
    Ok(())
}

pub fn write_ifd_entry<W: WriteBytesExt>(writer: &mut W, ifd_entry: &IfdEntry) -> std::io::Result<()> {
    writer.write_u16::<LittleEndian>(ifd_entry.tag)?;
    writer.write_u16::<LittleEndian>(ifd_entry.field_type)?;
    writer.write_u32::<LittleEndian>(ifd_entry.count)?;
    writer.write_u32::<LittleEndian>(ifd_entry.value_offset)?;
    Ok(())
}

pub fn save_32bit_grayscale_tiff(
    path: &Path, 
    normal: &Vec<Vec<f32>>, 
    size: usize
    ) -> std::io::Result<()> {
        
    let mut file = File::create(path)?;

    // Write the TIFF header
    write_tiff_header(&mut file, 8)?;

    // Write the IFD entries (image width, height, etc.)
    let ifd_entries = [
        // Image width
        IfdEntry {
            tag: 256,
            field_type: 4,
            count: 1,
            value_offset: size as u32,
        },
        // Image height
        IfdEntry {
            tag: 257,
            field_type: 4,
            count: 1,
            value_offset: size as u32,
        },
        // Bits per sample
        IfdEntry {
            tag: 258,
            field_type: 3,
            count: 1,
            value_offset: 32,
        },
        // Sample format
        IfdEntry {
            tag: 339,
            field_type: 3,
            count: 1,
            value_offset: 3,  // 1 for unsigned integer data
        },
        // Compression (no compression)
        IfdEntry {
            tag: 259,
            field_type: 3,
            count: 1,
            value_offset: 1,
        },
        // Photometric interpretation (black is zero)
        IfdEntry {
            tag: 262,
            field_type: 3,
            count: 1,
            value_offset: 1,
        },
        // Strip offsets
        IfdEntry {
            tag: 273,
            field_type: 4,
            count: 1,
            value_offset: 0x100,
        },
        // Samples per pixel
        IfdEntry {
            tag: 277,
            field_type: 3,
            count: 1,
            value_offset: 1,
        },
        // Rows per strip
        IfdEntry {
            tag: 278,
            field_type: 4,
            count: 1,
            value_offset: size as u32,
        },
        // Strip byte counts
        IfdEntry {
            tag: 279,
            field_type: 4,
            count: 1,
            value_offset: (size * size * 4) as u32,
        },
        // X resolution (placeholder)
        IfdEntry {
            tag: 282,
            field_type: 5,
            count: 1,
            value_offset: size as u32,
        },
        // Y resolution (placeholder)
        IfdEntry {
            tag: 283,
            field_type: 5,
            count: 1,
            value_offset: size as u32,
        },
    ];

    // Write the number of IFD entries and the entries themselves
    file.write_u16::<LittleEndian>(ifd_entries.len() as u16)?;
    for entry in &ifd_entries {
        write_ifd_entry(&mut file, entry)?;
    }

    // Write the next IFD offset (0, indicating the end of the file)
    file.write_u32::<LittleEndian>(0)?;

    // Write the image data (buffer) to the file
    // file.write_all(buffer)?;

    let buf_size = (size) * (size) * 4;
    let mut buf = vec![0_u8; buf_size];

    println!("loading into buffer...");
    for y in 0..size {
        for x in 0..size {
            let index = (y * size + x) * 4;
            //let value = (normal[x][y] * u32::MAX as f32).round() as u32;
            LittleEndian::write_f32(&mut buf[index..index+4], normal[x][y]);
        }   
    }   

    //println!("{:?}", &buf);
    println!("writing image data...");
    file.write_all(&buf)?;
    //writer.write_image_data(&buf)?;

    Ok(())
}


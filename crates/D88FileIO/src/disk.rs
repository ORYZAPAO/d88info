use std::io::{BufReader, Read};
use std::io::{Seek, SeekFrom};
use std::mem;

use crate::format::{D88_Header, D88_SectorHdr, MAX_SECTOR};

///
#[derive(Default, Debug)]
pub struct Sector {
    pub offset: u64,
    pub header: D88_SectorHdr,
    pub data: Vec<u8>,
}
impl Sector {
    /// Read Sector
    ///
    /// # Argument
    ///
    ///   * `reader` &mut BufReader<std::fs::File>
    ///
    /// # Return
    ///
    ///   * Ok(usize)  Sector Data Size (with Sector Header)
    ///   * Err(())    
    ///
    #[allow(clippy::result_unit_err)]
    pub fn preset(
        &mut self,
        reader: &mut BufReader<std::fs::File>,
        offset: u64,
    ) -> Result<u64, ()> {
        if offset == 0 {
            return Err(());
        }

        if reader.seek(SeekFrom::Start(offset)).is_err() {
            return Err(());
        }

        let mut buf: [u8; mem::size_of::<D88_SectorHdr>()] = [0; mem::size_of::<D88_SectorHdr>()]; // Header Buffer
        if let Ok(read_size) = reader.read(&mut buf) {
            if read_size != mem::size_of::<D88_SectorHdr>() {
                return Err(());
            }

            let d88_sector_header;
            unsafe {
                d88_sector_header =
                    mem::transmute::<[u8; mem::size_of::<D88_SectorHdr>()], D88_SectorHdr>(buf);
            }
            let sector_offset = offset + mem::size_of::<D88_SectorHdr>() as u64;

            let ret_sector_size =
                mem::size_of::<D88_SectorHdr>() + ((128 << d88_sector_header.sector_size) as usize);

            let mut sector_data: Vec<u8> = vec![0; d88_sector_header.size_of_data.into()];
            if reader.seek(SeekFrom::Start(sector_offset)).is_err() {
                return Err(());
            }
            if reader.read(&mut sector_data).is_err() {
                return Err(());
            }

            //
            self.offset = sector_offset;
            self.header = d88_sector_header;
            self.data = sector_data;

            Ok(ret_sector_size as u64)
        } else {
            Err(())
        }
    }
}

///
#[derive(Default, Debug)]
pub struct Track {
    pub number_of_sector: u16,
    pub sector_tbl: Vec<Sector>,
}

impl Track {
    /// Read Track
    ///
    /// # Argument
    ///
    ///   * `reader` &mut BufReader<std::fs::File>
    ///
    /// # Return
    ///
    ///   * Ok(usize)  Track Data Size
    ///   * Err(())    
    ///
    #[allow(clippy::result_unit_err)]
    pub fn preset(
        &mut self,
        reader: &mut BufReader<std::fs::File>,
        offset_: u64,
    ) -> Result<usize, ()> {
        if offset_ == 0 {
            return Err(());
        }
        let mut offset = offset_;

        let mut sec_count: u16 = 0;
        let mut track_size: usize = 0;

        #[allow(unused_assignments)]
        let mut number_of_sector: u16 = 0;

        loop {
            let mut sector = Sector::default();

            if let Ok(sec_size) = sector.preset(reader, offset) {
                track_size += sec_size as usize;

                number_of_sector = sector.header.number_of_sec;

                self.sector_tbl.push(sector);

                sec_count += 1;
                if (sec_count >= number_of_sector) || (sec_count >= MAX_SECTOR) {
                    break;
                }

                //
                offset += sec_size;
            } else {
                return Err(());
            }
        }

        self.number_of_sector = number_of_sector;
        Ok(track_size)
    }

    /// Sector Table Sort by Sector Order
    ///
    /// # Argument
    ///
    ///   * (none)
    ///
    /// # Return
    ///
    ///   * (none)
    ///
    pub fn sector_sort(&mut self) {
        // Sort Sector Table
        self.sector_tbl
            .sort_by(|a: &Sector, b: &Sector| a.header.sector.cmp(&b.header.sector));
    }

    /// Sector Table Sort by File Offset Order
    ///
    /// # Argument
    ///
    ///   * (none)
    ///
    /// # Return
    ///
    ///   * (none)
    ///
    pub fn file_offset_sort(&mut self) {
        // Sort Sector Table
        self.sector_tbl
            .sort_by(|a: &Sector, b: &Sector| a.offset.cmp(&b.offset));
    }
}

///
#[derive(Default, Debug)]
pub struct Disk {
    pub header: D88_Header,
    pub track_tbl: Vec<Track>,
}

impl Disk {
    /// Read Disk
    ///
    /// # Argument
    ///
    ///   * `reader` &mut BufReader<std::fs::File>
    ///
    /// # Return
    ///
    ///   * Ok(usize)  Disk Size
    ///   * Err(())    
    ///
    #[allow(clippy::result_unit_err)]
    pub fn preset(&mut self, reader: &mut BufReader<std::fs::File>) -> Result<usize, ()> {
        if reader.seek(SeekFrom::Start(0)).is_err() {
            return Err(());
        }

        let mut buf: [u8; mem::size_of::<D88_Header>()] = [0; mem::size_of::<D88_Header>()]; // Header Buffer
        if let Ok(read_size) = reader.read(&mut buf) {
            //
            if read_size != mem::size_of::<D88_Header>() {
                return Err(());
            }

            let header;
            unsafe {
                header = mem::transmute::<[u8; mem::size_of::<D88_Header>()], D88_Header>(buf);
            }
            self.header = header;

            self.preset_track(reader) // return Ok(disk_size :usize)
        } else {
            Err(())
        }
        //
    }

    /// Read Track and Sector
    ///
    /// # Argument
    ///
    ///   * `reader` &mut BufReader<std::fs::File>
    ///
    /// # Return
    ///
    ///   * Ok(usize)  Disk Size
    ///   * Err(())    
    ///
    #[allow(clippy::result_unit_err)]
    pub fn preset_track(&mut self, reader: &mut BufReader<std::fs::File>) -> Result<usize, ()> {
        let mut disk_size: usize = 0;

        for track_offset in self.header.track_tbl {
            let mut track = Track::default();

            if let Ok(track_size) = track.preset(reader, track_offset as u64) {
                disk_size += track_size as usize;
            } else {
                break;
            }
            self.track_tbl.push(track);
        }

        if disk_size == 0 {
            return Err(());
        }
        //assert_eq!(num_of_track, 80); // 2D Disk
        Ok(disk_size)
    }
}

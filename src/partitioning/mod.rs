use std::borrow::{Borrow, BorrowMut};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use loopdev::{LoopControl, LoopDevice};
use mbrman::{MBRPartitionEntry, BOOT_ACTIVE, BOOT_INACTIVE, CHS, MBR};
use size::Size;

mod lvm;

pub fn allocate_image(image_path: String, size: Size) -> LoopDevice {
    // needs to do the following

    let path = Rc::new(PathBuf::from(image_path));
    let mut file_handle = allocate_file(path.as_ref(), size);
    create_partition_tables(&mut file_handle, Size::from_mebibytes(250));
    file_handle.flush().unwrap();
    let backing_device = Rc::new(setup_loop(path.as_ref()));

    lvm::logical_volume_creation(backing_device.as_ref());

    Rc::try_unwrap(backing_device).unwrap()
}

fn allocate_file(path: &Path, size: Size) -> File {
    let handle = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    handle.set_len(size.bytes() as u64).unwrap();
    handle
}

fn setup_loop(path: &Path) -> LoopDevice {
    let lc = LoopControl::open().unwrap();
    let device = lc.next_free().unwrap();
    device
        .with()
        .part_scan(true)
        .attach(path.to_str().unwrap())
        .unwrap();
    device
}

fn create_partition_tables(device: &mut File, boot_size: Size) {
    let sector_size = 512;
    let mut mbr = Rc::new(MBR::new_from(device, sector_size, [0xff; 4]).unwrap());

    let boot_sectors = boot_bytes_to_sectors(boot_size, sector_size);

    Rc::get_mut(&mut mbr).unwrap()[1] = MBRPartitionEntry {
        boot: BOOT_ACTIVE,

        first_chs: CHS::empty(),
        // w95 fat32 (LBA)
        sys: 0xc,
        last_chs: CHS::empty(),
        starting_lba: 4 * sector_size,
        sectors: boot_sectors,
    };

    let root_partition = get_next_lba_and_remainder(Rc::as_ref(&mbr));

    Rc::get_mut(&mut mbr).unwrap()[2] = MBRPartitionEntry {
        boot: BOOT_INACTIVE,
        first_chs: CHS::empty(),
        // lvm partition id
        sys: 0x8e,
        last_chs: CHS::empty(),
        starting_lba: root_partition.0,
        sectors: root_partition.1,
    };

    Rc::get_mut(&mut mbr).unwrap().write_into(device).unwrap();
}

fn boot_bytes_to_sectors(input: Size, sector_size: u32) -> u32 {
    let mut size = input.bytes() as u32 / sector_size;
    let remainder = input.bytes() as u32 % sector_size;
    if remainder != 0 {
        size += remainder;
    }
    size
}

fn get_next_lba_and_remainder(disk: &MBR) -> (u32, u32) {
    let leftover_sectors = disk.disk_size - disk[1].starting_lba - disk[1].sectors;
    let starting_lba = disk[1].starting_lba + disk[1].sectors;
    (starting_lba, leftover_sectors)
}

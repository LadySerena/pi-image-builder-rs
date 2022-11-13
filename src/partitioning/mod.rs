use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use loopdev::{LoopControl, LoopDevice};
use mbrman::{MBRPartitionEntry, BOOT_ACTIVE, BOOT_INACTIVE, CHS};
use size::Size;

pub fn allocate_image(image_path: String, size: Size) {
    // needs to do the following

    let path = Rc::new(PathBuf::from(image_path));
    let mut file_handle = allocate_file(path.as_ref(), size);
    // let backing_device = setup_loop(path.as_ref());
    create_partition_tables(&mut file_handle, Size::from_mebibytes(250));
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
    device.attach_file(path.to_str().unwrap()).unwrap();
    device
}

fn create_partition_tables(device: &mut File, boot_size: Size) {
    let sector_size = 512;
    let mut mbr = mbrman::MBR::new_from(device, sector_size, [0xff; 4]).unwrap();

    let used_sectors = boot_size.bytes() / i64::from(sector_size);

    mbr[1] = MBRPartitionEntry {
        boot: BOOT_ACTIVE,

        first_chs: CHS::empty(),
        // w95 fat32 (LBA)
        sys: 0xc,
        last_chs: CHS::empty(),
        starting_lba: 4 * sector_size,
        sectors: used_sectors as u32,
    };

    let leftover_sectors = mbr.disk_size - (4 * sector_size) - mbr[1].sectors;

    mbr[2] = MBRPartitionEntry {
        boot: BOOT_INACTIVE,
        first_chs: CHS::empty(),
        // lvm partition id
        sys: 0x8e,
        last_chs: CHS::empty(),
        starting_lba: used_sectors as u32 + (4 * sector_size),
        sectors: leftover_sectors,
    };

    mbr.write_into(device).unwrap();
}

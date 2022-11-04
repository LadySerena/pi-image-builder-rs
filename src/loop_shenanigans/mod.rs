use loopdev::LoopControl;

// TODO need to return the loop device
// TODO figure out how to do this sans sudo
pub fn map_image_to_loop_device(input: String) {
    let lc = LoopControl::open().unwrap();
    let ld = lc.next_free().unwrap();
    ld.attach_file(input.as_str()).unwrap();
}

use crate::filesystem::filesystem::create_image_file_systems;
use crate::partitioning::RuntimeImageInfo;

mod filesystem;

pub fn create(image: &RuntimeImageInfo) {
    create_image_file_systems(image)
}

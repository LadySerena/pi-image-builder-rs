use crate::filesystem::filesystem::create_image_file_systems;
use crate::partitioning::ImageInfo;

mod filesystem;

pub fn create(image: &ImageInfo) {
    create_image_file_systems(image)
}

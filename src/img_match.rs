use crate::BotResult;

const HAMMING_TOLERANCE: u64 = 2;

pub struct ImageMatcher {
    hasher: img_hash::Hasher,
}

impl ImageMatcher {
    pub fn new() -> Self {
        let hasher = img_hash::HasherConfig::new()
            .hash_alg(img_hash::HashAlg::DoubleGradient)
            .hash_size(14, 14)
            .preproc_dct()
            .to_hasher();
        Self { hasher }
    }

    pub fn hash(&self, vk_image: &[u8]) -> BotResult<img_hash::ImageHash> {
        // Fun fact: VK image previews are JPEGs regardless of the format of the original pic
        let image = image::load_from_memory_with_format(vk_image, image::ImageFormat::Jpeg)?;
        let image_hash = self.hasher.hash_image(&image);
        Ok(image_hash)
    }

    pub fn matches(expected: [u8; 14], hash: img_hash::ImageHash) -> bool {
        let dist = hamming::distance(&expected, hash.as_bytes());
        dist <= HAMMING_TOLERANCE
    }
}

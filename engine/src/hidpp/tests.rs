#[cfg(test)]
mod tests {
    use crate::hidpp::protocol::*;

    #[test]
    fn test_feature_constants() {
        assert_eq!(FEAT_IROOT, 0x0000);
        assert_eq!(FEAT_REPROG_V4, 0x1B04);
        assert_eq!(FEAT_ADJ_DPI, 0x2201);
        assert_eq!(FEAT_BACKLIGHT2, 0x1982);
    }
}

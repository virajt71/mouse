pub fn detect_receivers() -> (bool, bool) {
    let mut unifying = false;
    let mut bolt = false;

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/bus/usb/devices") {
            for entry in entries.flatten() {
                let path = entry.path();
                let vendor_path = path.join("idVendor");
                let product_path = path.join("idProduct");

                if let (Ok(vendor), Ok(product)) = (
                    std::fs::read_to_string(vendor_path),
                    std::fs::read_to_string(product_path),
                ) {
                    let vendor = vendor.trim();
                    let product = product.trim();

                    // Logitech Vendor ID is 046d
                    if vendor == "046d" {
                        // Product IDs for Logitech receivers:
                        // c52b: Unifying Receiver
                        // c532: Unifying Receiver
                        // c534: Nano Receiver
                        // c539: Lightspeed Receiver
                        if product == "c52b"
                            || product == "c532"
                            || product == "c534"
                            || product == "c539"
                        {
                            unifying = true;
                        } else if product == "c542" || product == "c548" {
                            // c542, c548: Bolt Receiver
                            bolt = true;
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Default to true on other platforms for design-time rendering / fallback
        unifying = true;
        bolt = true;
    }

    (unifying, bolt)
}

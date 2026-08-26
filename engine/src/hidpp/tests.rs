#[cfg(test)]
mod tests {
    use crate::hidpp::protocol::*;

    #[test]
    fn test_feature_constants() {
        assert_eq!(FEAT_IROOT, 0x0000);
        assert_eq!(FEAT_REPROG_V4, 0x1B04);
        assert_eq!(FEAT_ADJ_DPI, 0x2201);
        assert_eq!(FEAT_BACKLIGHT2, 0x1982);
        assert_eq!(FEAT_UNIFIED_BATTERY, 0x1004);
        assert_eq!(FEAT_BATTERY_STATUS, 0x1000);
        assert_eq!(FEAT_BATTERY_VOLTAGE, 0x1001);
    }

    #[test]
    fn test_parse_unified_battery() {
        // payload: [percentage=80, level=Good, status=Charging(1)]
        let (pct, status) = parse_unified_battery(&[80, 0x04, 1]);
        assert_eq!(pct, 80);
        assert_eq!(status, "charging");
    }

    #[test]
    fn test_unified_battery_status_labels() {
        assert_eq!(unified_battery_status_label(0), "discharging");
        assert_eq!(unified_battery_status_label(1), "charging");
        assert_eq!(unified_battery_status_label(3), "full");
        assert_eq!(unified_battery_status_label(4), "charging_slow");
        assert_eq!(unified_battery_status_label(7), "charging_error");
        assert_eq!(unified_battery_status_label(99), "unknown");
    }

    #[test]
    fn test_legacy_battery_status_labels() {
        assert_eq!(legacy_status_label(0), "discharging");
        assert_eq!(legacy_status_label(1), "recharging");
        assert_eq!(legacy_status_label(3), "full");
        assert_eq!(legacy_status_label(7), "other");
        assert_eq!(legacy_status_label(55), "unknown");
    }

    #[test]
    fn test_parse_unified_battery_clamps_percentage() {
        // Firmware could report >100 on some devices; clamp defensively.
        let (pct, _) = parse_unified_battery(&[120, 0x08, 0]);
        assert_eq!(pct, 100);
    }
}

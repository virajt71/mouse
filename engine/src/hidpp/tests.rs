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



    // ---- level byte was previously ignored; now decoded ----
    #[test]
    fn test_parse_unified_battery_full_population() {
        // payload: [percentage=80, level=Good(4), status=Charging(1)]
        let (pct, level, status) = parse_unified_battery_full(&[80, 4, 1]).unwrap();
        assert_eq!(pct, 80);
        assert_eq!(level, "good");
        assert_eq!(status, "charging");
    }

    #[test]
    fn test_unified_battery_level_labels() {
        assert_eq!(unified_battery_level_label(1), "critical");
        assert_eq!(unified_battery_level_label(2), "low");
        assert_eq!(unified_battery_level_label(4), "good");
        assert_eq!(unified_battery_level_label(8), "full");
        // unknown level values (vendor extensions, stale data) land as unknown
        assert_eq!(unified_battery_level_label(0), "unknown_level");
        assert_eq!(unified_battery_level_label(3), "unknown_level");
        assert_eq!(unified_battery_level_label(99), "unknown_level");
    }

    #[test]
    fn test_parse_unified_battery_full_short_payload() {
        // Too-short payloads surface as None rather than panicking.
        assert!(parse_unified_battery_full(&[]).is_none());
        assert!(parse_unified_battery_full(&[80]).is_none());
        assert!(parse_unified_battery_full(&[80, 4]).is_none());
    }
}


#[cfg(test)]
mod voltage_tests {
    use crate::hidpp::protocol::*;

    #[test]
    fn test_voltage_battery_status_labels() {
        assert_eq!(voltage_battery_status_label(0), "discharging");
        assert_eq!(voltage_battery_status_label(1), "charging");
        assert_eq!(voltage_battery_status_label(2), "charging_fast");
        assert_eq!(voltage_battery_status_label(3), "full");
        assert_eq!(voltage_battery_status_label(4), "charging_slow");
        assert_eq!(voltage_battery_status_label(5), "invalid_battery");
        assert_eq!(voltage_battery_status_label(7), "charging_error");
        assert_eq!(voltage_battery_status_label(99), "unknown");
    }

    #[test]
    fn test_parse_voltage_battery() {
        let (mv, status) = parse_voltage_battery(&[0x0F, 0xA0, 1]).unwrap();
        assert_eq!(mv, 0x0FA0); // 4000 mV
        assert_eq!(status, "charging");
    }

    #[test]
    fn test_parse_voltage_battery_short_payload() {
        assert!(parse_voltage_battery(&[]).is_none());
        assert!(parse_voltage_battery(&[0x0F]).is_none());
    }

    #[test]
    fn test_voltage_to_battery_percent_boundaries() {
        assert_eq!(voltage_to_battery_percent(4200), 100);
        assert_eq!(voltage_to_battery_percent(4100), 90);
        assert_eq!(voltage_to_battery_percent(4000), 80);
        assert_eq!(voltage_to_battery_percent(3000), 0);
        assert_eq!(voltage_to_battery_percent(2900), 0);
    }

    #[test]
    fn test_voltage_to_battery_percent_midrange() {
        let pct = voltage_to_battery_percent(3850);
        assert!(pct >= 60 && pct <= 70, "unexpected pct {pct} for 3850mV");
    }
}

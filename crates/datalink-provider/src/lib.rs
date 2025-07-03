//! Real AIS and GPS Datalink Providers
//! 
//! This crate provides real-world implementations of AIS and GPS datalink providers
//! that can connect to actual data sources such as:
//! - Serial ports (for direct AIS/GPS receiver connections)
//! - TCP/UDP network connections (for networked AIS/GPS data)
//! - File-based AIS/GPS data replay

mod ais;
mod gps;

// Re-export the main types for external use
pub use ais::{AisDataLinkProvider, AisSourceConfig};
pub use gps::{GpsDataLinkProvider, GpsSourceConfig};

use datalink::{DataLinkConfig, DataLinkReceiver, DataLinkStatus};




#[cfg(test)]
mod tests {
    use super::*;
    use datalink::DataLinkConfig;
    use crate::ais::{AisDataLinkProvider, AisSourceConfig};
    use crate::gps::{GpsDataLinkProvider, GpsSourceConfig};

    #[test]
    fn test_ais_provider_creation() {
        let provider = AisDataLinkProvider::new();
        assert!(matches!(DataLinkReceiver::status(&provider), DataLinkStatus::Disconnected));
    }

    #[test]
    fn test_parse_source_config_serial() {
        let config = DataLinkConfig::new("serial".to_string())
            .with_parameter("connection_type".to_string(), "serial".to_string())
            .with_parameter("port".to_string(), "/dev/ttyUSB0".to_string())
            .with_parameter("baud_rate".to_string(), "38400".to_string());

        let source_config = AisDataLinkProvider::parse_source_config(&config).unwrap();

        match source_config {
            AisSourceConfig::Serial { port, baud_rate } => {
                assert_eq!(port, "/dev/ttyUSB0");
                assert_eq!(baud_rate, 38400);
            }
            _ => panic!("Expected Serial configuration"),
        }
    }

    #[test]
    fn test_parse_source_config_tcp() {
        let config = DataLinkConfig::new("tcp".to_string())
            .with_parameter("connection_type".to_string(), "tcp".to_string())
            .with_parameter("host".to_string(), "localhost".to_string())
            .with_parameter("port".to_string(), "12345".to_string());

        let source_config = AisDataLinkProvider::parse_source_config(&config).unwrap();

        match source_config {
            AisSourceConfig::Tcp { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 12345);
            }
            _ => panic!("Expected TCP configuration"),
        }
    }

    #[test]
    fn test_parse_ais_sentence() {
        let sentence = "!AIVDM,1,1,,A,15M8J7001G?UJH@E=4R0S>0@0<0M,0*7B";
        let message = AisDataLinkProvider::parse_ais_sentence(sentence).unwrap();

        assert_eq!(message.message_type, "AIS_SENTENCE");
        assert_eq!(message.source_id, "AIS_RECEIVER");
        assert_eq!(message.get_data("sentence_type"), Some(&"!AIVDM".to_string()));
        assert_eq!(message.get_data("payload"), Some(&"15M8J7001G?UJH@E=4R0S>0@0<0M".to_string()));
    }

    #[test]
    fn test_invalid_ais_sentence() {
        let sentence = "This is not an AIS sentence";
        let message = AisDataLinkProvider::parse_ais_sentence(sentence);
        assert!(message.is_none());
    }

    // GPS Provider Tests
    #[test]
    fn test_gps_provider_creation() {
        let provider = GpsDataLinkProvider::new();
        assert!(matches!(DataLinkReceiver::status(&provider), DataLinkStatus::Disconnected));
    }

    #[test]
    fn test_parse_gps_source_config_serial() {
        let config = DataLinkConfig::new("serial".to_string())
            .with_parameter("connection_type".to_string(), "serial".to_string())
            .with_parameter("port".to_string(), "/dev/ttyUSB0".to_string())
            .with_parameter("baud_rate".to_string(), "9600".to_string());

        let source_config = GpsDataLinkProvider::parse_source_config(&config).unwrap();

        match source_config {
            GpsSourceConfig::Serial { port, baud_rate } => {
                assert_eq!(port, "/dev/ttyUSB0");
                assert_eq!(baud_rate, 9600);
            }
            _ => panic!("Expected Serial configuration"),
        }
    }

    #[test]
    fn test_parse_gps_source_config_tcp() {
        let config = DataLinkConfig::new("tcp".to_string())
            .with_parameter("connection_type".to_string(), "tcp".to_string())
            .with_parameter("host".to_string(), "gps.example.com".to_string())
            .with_parameter("port".to_string(), "2947".to_string());

        let source_config = GpsDataLinkProvider::parse_source_config(&config).unwrap();

        match source_config {
            GpsSourceConfig::Tcp { host, port } => {
                assert_eq!(host, "gps.example.com");
                assert_eq!(port, 2947);
            }
            _ => panic!("Expected TCP configuration"),
        }
    }

    #[test]
    fn test_parse_gps_gga_sentence() {
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence).unwrap();

        assert_eq!(message.message_type, "GPS_SENTENCE");
        assert_eq!(message.source_id, "GPS_RECEIVER");
        assert_eq!(message.get_data("sentence_type"), Some(&"$GPGGA".to_string()));
        assert_eq!(message.get_data("time"), Some(&"123519".to_string()));
        assert_eq!(message.get_data("latitude"), Some(&"4807.038".to_string()));
        assert_eq!(message.get_data("lat_direction"), Some(&"N".to_string()));
        assert_eq!(message.get_data("longitude"), Some(&"01131.000".to_string()));
        assert_eq!(message.get_data("lon_direction"), Some(&"E".to_string()));
        assert_eq!(message.get_data("fix_quality"), Some(&"1".to_string()));
        assert_eq!(message.get_data("satellites"), Some(&"08".to_string()));
        assert_eq!(message.get_data("hdop"), Some(&"0.9".to_string()));
        assert_eq!(message.get_data("altitude"), Some(&"545.4".to_string()));
    }

    #[test]
    fn test_parse_gps_rmc_sentence() {
        let sentence = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence).unwrap();

        assert_eq!(message.message_type, "GPS_SENTENCE");
        assert_eq!(message.source_id, "GPS_RECEIVER");
        assert_eq!(message.get_data("sentence_type"), Some(&"$GPRMC".to_string()));
        assert_eq!(message.get_data("time"), Some(&"123519".to_string()));
        assert_eq!(message.get_data("status"), Some(&"A".to_string()));
        assert_eq!(message.get_data("latitude"), Some(&"4807.038".to_string()));
        assert_eq!(message.get_data("speed"), Some(&"022.4".to_string()));
        assert_eq!(message.get_data("course"), Some(&"084.4".to_string()));
        assert_eq!(message.get_data("date"), Some(&"230394".to_string()));
    }

    #[test]
    fn test_parse_gps_gll_sentence() {
        let sentence = "$GPGLL,4916.45,N,12311.12,W,225444,A,*1D";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence).unwrap();

        assert_eq!(message.message_type, "GPS_SENTENCE");
        assert_eq!(message.source_id, "GPS_RECEIVER");
        assert_eq!(message.get_data("sentence_type"), Some(&"$GPGLL".to_string()));
        assert_eq!(message.get_data("latitude"), Some(&"4916.45".to_string()));
        assert_eq!(message.get_data("lat_direction"), Some(&"N".to_string()));
        assert_eq!(message.get_data("longitude"), Some(&"12311.12".to_string()));
        assert_eq!(message.get_data("lon_direction"), Some(&"W".to_string()));
        assert_eq!(message.get_data("time"), Some(&"225444".to_string()));
        assert_eq!(message.get_data("status"), Some(&"A".to_string()));
    }

    #[test]
    fn test_parse_gnss_sentence() {
        let sentence = "$GNGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence).unwrap();

        assert_eq!(message.message_type, "GPS_SENTENCE");
        assert_eq!(message.source_id, "GPS_RECEIVER");
        assert_eq!(message.get_data("sentence_type"), Some(&"$GNGGA".to_string()));
        assert_eq!(message.get_data("latitude"), Some(&"4807.038".to_string()));
    }

    #[test]
    fn test_invalid_gps_sentence() {
        let sentence = "This is not a GPS sentence";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence);
        assert!(message.is_none());
    }

    #[test]
    fn test_invalid_gps_sentence_no_dollar() {
        let sentence = "GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence);
        assert!(message.is_none());
    }

    #[test]
    fn test_unsupported_gps_sentence() {
        let sentence = "$GPXXX,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
        let message = GpsDataLinkProvider::parse_gps_sentence(sentence);
        assert!(message.is_none());
    }
}

//! 首次升级时关闭此前被强制暂停的云联想，之后保留用户主动选择。

use std::fs;
use std::io;
use std::path::Path;

use qingjian_core::storage::write_atomic_str;
use toml_edit::{DocumentMut, table, value};

pub(crate) fn cloud_controls(path: &Path) -> io::Result<()> {
    let marker = path.with_file_name("cloud-controls-v3.migrated");
    if marker.exists() {
        return Ok(());
    }
    let source = fs::read_to_string(path)?;
    let mut document: DocumentMut = source
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let backup = path.with_file_name("config-before-v3.toml");
    if !backup.exists() {
        fs::copy(path, backup)?;
    }
    if !document.get("predict").is_some_and(|item| item.is_table()) {
        document["predict"] = table();
    }
    document["predict"]["enabled"] = value(false);
    document["predict"]["automatic"] = value(false);
    write_atomic_str(path, &document.to_string())?;
    write_atomic_str(&marker, "3\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "qingjian-v3-migration-{}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn upgrade_backs_up_legacy_cloud_flags_and_preserves_service_settings() {
        let dir = directory("upgrade");
        let path = dir.join("config.toml");
        let old = "# 用户配置\n[predict]\nenabled=true\nautomatic=true\nmodel='local-test'\napi_key='fake-local-test-key'\n";
        fs::write(&path, old).unwrap();
        cloud_controls(&path).unwrap();
        let config = qingjian_platform::Config::load(&path).unwrap();
        assert!(!config.predict.enabled);
        assert!(!config.predict.automatic);
        assert_eq!(config.predict.model, "local-test");
        assert_eq!(
            config.predict.api_key.as_deref(),
            Some("fake-local-test-key")
        );
        assert_eq!(
            fs::read_to_string(dir.join("config-before-v3.toml")).unwrap(),
            old
        );
        assert!(fs::read_to_string(&path).unwrap().contains("# 用户配置"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_later_launch_keeps_the_users_explicit_cloud_preference() {
        let dir = directory("keep");
        let path = dir.join("config.toml");
        fs::write(&path, "[predict]\nenabled=true\nautomatic=true\n").unwrap();
        cloud_controls(&path).unwrap();
        qingjian_platform::Config::set_value(&path, "predict", "enabled", true).unwrap();
        qingjian_platform::Config::set_value(&path, "predict", "automatic", true).unwrap();
        cloud_controls(&path).unwrap();
        let config = qingjian_platform::Config::load(&path).unwrap();
        assert!(config.predict.enabled && config.predict.automatic);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn malformed_configuration_is_preserved_without_a_migration_marker() {
        let dir = directory("broken");
        let path = dir.join("config.toml");
        fs::write(&path, "[broken").unwrap();
        assert!(cloud_controls(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "[broken");
        assert!(!dir.join("cloud-controls-v3.migrated").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}

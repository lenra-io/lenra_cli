#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let mut context = CommandContext::default();
        context.config_path = PathBuf::from("test_config.toml");
        let app = context.load_config().unwrap();
        assert_eq!(app.name, "test_app");
    }

    #[test]
    fn test_resolve_path() {
        let mut context = CommandContext::default();
        context.config_path = PathBuf::from("/home/user/lenra/config.toml");
        let path = PathBuf::from("src/main.rs");
        let resolved_path = context.resolve_path(&path);
        assert_eq!(
            resolved_path,
            PathBuf::from("/home/user/lenra/src/main.rs")
        );
    }

    #[test]
    fn test_get_app_workdir() {
        let mut context = CommandContext::default();
        context.config_path = PathBuf::from("/home/user/lenra/config.toml");
        let workdir = context.get_app_workdir();
        assert_eq!(workdir, PathBuf::from("/home/user/lenra"));
    }

    #[test]
    fn test_get_app_path_config() {
        let mut context = CommandContext::default();
        let app = Application {
            name: "test_app".to_string(),
            path: Some(PathBuf::from("test_app")),
            ..Default::default()
        };
        context.config = Some(app);
        let app_path = context.get_app_path_config().unwrap();
        assert_eq!(app_path, PathBuf::from("test_app"));
    }
}
pub fn canonicalize(path: impl AsRef<std::path::Path>) -> std::io::Result<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        dunce::canonicalize(path).and_then(|p| {
            let p = p.display().to_string();
            let prefixed = format!("/{}", p.replace("\\", "/"));
            Ok(prefixed.into())
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::fs::canonicalize(path)
    }
}

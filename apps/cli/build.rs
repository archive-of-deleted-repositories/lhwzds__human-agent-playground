use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("../..").canonicalize()?;
    let dist = root.join("apps/web/dist");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated = out_dir.join("static_assets.rs");

    println!("cargo:rerun-if-changed={}", dist.display());

    let assets = if dist.join("index.html").is_file() {
        let mut assets = Vec::new();
        collect_assets(&dist, &dist, &mut assets)?;
        assets
    } else {
        let fallback_dir = out_dir.join("fallback-web");
        fs::create_dir_all(&fallback_dir)?;
        let fallback = fallback_dir.join("index.html");
        fs::write(
            &fallback,
            "<!doctype html><html><body><h1>Human Agent Playground</h1><p>Run npm run build before building hap to embed the web UI.</p></body></html>",
        )?;
        vec![("index.html".to_string(), fallback)]
    };

    let mut code = String::from(
        "pub struct StaticAsset {\n    pub path: &'static str,\n    pub content_type: &'static str,\n    pub bytes: &'static [u8],\n}\n\npub static STATIC_ASSETS: &[StaticAsset] = &[\n",
    );

    for (path, file) in assets {
        println!("cargo:rerun-if-changed={}", file.display());
        code.push_str(&format!(
            "    StaticAsset {{ path: {:?}, content_type: {:?}, bytes: include_bytes!(r#\"{}\"#) }},\n",
            path,
            content_type_for(&path),
            file.display()
        ));
    }
    code.push_str("];\n");
    fs::write(generated, code)?;
    Ok(())
}

fn collect_assets(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_assets(root, &path, out)?;
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        out.push((rel, path));
    }

    Ok(())
}

fn content_type_for(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|value| value.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        Some("ico") => "image/x-icon",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

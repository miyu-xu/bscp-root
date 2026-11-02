// Copyright 2026, The Android Open Source Project
// Licensed under the Apache License, Version 2.0.
//! Desktop preinstalled-artifact discovery. This is not Android PackageManager
//! staging, signature verification, or classpath discovery.
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub(crate) struct FactoryApex {
    pub name: String,
    pub path: PathBuf,
    pub source_time_seconds: u64,
    pub logical_path: PathBuf,
    pub partition: String,
}

/// Preserve the existing host-root configuration. Observation errors are not
/// converted into absence; prefix matching uses path components.
pub(crate) fn resolve_host_path(path: &Path) -> Result<PathBuf> {
    vmconfig::resolve_host_path(path)
}

pub(crate) fn check_explicit_modules<'a>(normal: impl Iterator<Item=&'a str>, tenants: impl Iterator<Item=&'a str>) -> Result<()> {
    if normal.chain(tenants).any(|name|name=="{CLASSPATH}") {
        bail!("Desktop Host requires explicit APEX module names; classpath discovery is unavailable");
    }
    Ok(())
}

type FactoryCandidate = (String, PathBuf, PathBuf, u64, bool);
fn discover_factory(roots: &[(PathBuf, PathBuf)]) -> Result<Vec<FactoryCandidate>> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    for (root, logical) in roots {
        let reader = match fs::read_dir(root) {
            Ok(v) => v,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e).with_context(|| format!("Reading {}", root.display())),
        };
        for entry in reader {
            let entry = entry?;
            let source = entry.path();
            let extension = source.extension().and_then(|s| s.to_str());
            if !matches!(extension, Some("apex" | "capex")) { continue; }
            if !entry.file_type()?.is_file() { bail!("Factory artifact is not a regular file: {}", source.display()); }
            let name = source.file_stem().and_then(|s| s.to_str()).context("Invalid APEX filename")?;
            if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b,b'.'|b'_'|b'-')) {
                bail!("Invalid APEX module name");
            }
            if !seen.insert(name.to_owned()) { bail!("Duplicate active factory APEX {name}"); }
            let metadata = entry.metadata()?;
            entries.push((name.to_owned(),source.clone(),logical.join(entry.file_name()),
                          metadata.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
                          extension == Some("capex")));
        }
    }
    if entries.is_empty() { bail!("Desktop product contains no factory APEX files"); }
    entries.sort_by(|a,b| (&a.0,&a.2).cmp(&(&b.0,&b.2)));
    Ok(entries)
}

fn logical_partition(logical: &Path) -> Result<String> {
    let parent=logical.parent().and_then(Path::parent).context("Invalid logical APEX partition")?;
    let partition=parent.file_name().and_then(|p|p.to_str()).context("Invalid partition name")?;
    if parent.parent()!=Some(Path::new("/")) || !matches!(partition,"system"|"system_ext"|"product"|"vendor"|"odm") {
        bail!("Unsupported logical APEX partition");
    }
    Ok(partition.to_owned())
}

/// Identity metadata lookup must not extract CAPEX or create a temporary output.
pub(crate) fn factory_partition(roots: &[(PathBuf,PathBuf)], name: &str) -> Result<String> {
    let entries=discover_factory(roots)?;
    let entry=entries.iter().find(|e|e.0==name).context("APEX module not found")?;
    logical_partition(&entry.2)
}

pub(crate) fn scan_factory(roots: &[(PathBuf, PathBuf)], temporary: &Path) -> Result<Vec<FactoryApex>> {
    let entries=discover_factory(roots)?;
    let cache = temporary.join("factory-apexes");
    if entries.iter().any(|e| e.4) { fs::create_dir(&cache).context("Creating exclusive factory APEX output")?; }
    let mut result = Vec::new();
    for (name,source,logical,source_time_seconds,compressed) in entries {
        let path = if compressed {
            let mut archive = zip::ZipArchive::new(File::open(&source)?)?;
            let mut member = archive.by_name("original_apex").context("CAPEX missing original_apex")?;
            let output_path = cache.join(format!("{name}.apex"));
            let mut output = OpenOptions::new().write(true).create_new(true).open(&output_path)?;
            io::copy(&mut member,&mut output)?;
            output.sync_all()?;
            output_path
        } else { source };
        let partition = logical_partition(&logical)?;
        result.push(FactoryApex{name,path,source_time_seconds,logical_path:logical,partition});
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn root(name: &str) -> PathBuf {
        let p=std::env::temp_dir().join(format!("a17-apex-{}-{name}",std::process::id()));
        fs::create_dir(&p).unwrap(); p
    }
    #[test]
    fn stable_factory_order_and_original_capex_time() {
        let p=root("stable"); let product=p.join("product"); fs::create_dir(&product).unwrap();
        fs::write(product.join("z.apex"),b"apex fixture").unwrap();
        let capex=product.join("a.capex");
        let mut zip=zip::ZipWriter::new(File::create(&capex).unwrap());
        zip.start_file("original_apex",zip::write::FileOptions::default()).unwrap();
        zip.write_all(b"embedded fixture").unwrap();zip.finish().unwrap();
        let original_time=fs::metadata(&capex).unwrap().modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let mut outcomes=Vec::new();
        for boot in ["one","two"] {
            let tmp=p.join(boot);fs::create_dir(&tmp).unwrap();
            let rows=scan_factory(&[(product.clone(),"/system/apex".into())],&tmp).unwrap();
            assert_eq!(rows.iter().map(|x|x.name.as_str()).collect::<Vec<_>>(),["a","z"]);
            assert_eq!(rows[0].source_time_seconds,original_time);
            assert_eq!(rows[0].partition,"system");assert_eq!(rows[0].logical_path,Path::new("/system/apex/a.capex"));
            assert_eq!(fs::read(&rows[0].path).unwrap(),b"embedded fixture");
            outcomes.push(rows[0].source_time_seconds);
        }
        assert_eq!(outcomes[0],outcomes[1]);fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn invalid_duplicate_missing_member_and_existing_output_fail() {
        let p=root("errors");let a=p.join("a");let b=p.join("b");fs::create_dir(&a).unwrap();fs::create_dir(&b).unwrap();
        let roots=vec![(a.clone(),"/system/apex".into()),(b.clone(),"/system_ext/apex".into())];
        assert!(scan_factory(&roots,&p).is_err());
        fs::write(a.join("x.apex"),b"a").unwrap();fs::write(b.join("x.apex"),b"b").unwrap();
        assert!(scan_factory(&roots,&p).is_err());fs::remove_file(b.join("x.apex")).unwrap();
        let mut zip=zip::ZipWriter::new(File::create(a.join("bad.capex")).unwrap());zip.start_file("wrong",zip::write::FileOptions::default()).unwrap();zip.finish().unwrap();
        assert!(scan_factory(&roots,&p).is_err());assert!(scan_factory(&roots,&p).is_err());
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn wrong_root_type_is_not_absence() {
        let p=root("root-type");let f=p.join("file");fs::write(&f,b"x").unwrap();
        assert!(scan_factory(&[(f,"/system/apex".into())],&p).is_err());fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn partition_lookup_does_not_decompress_or_create_outputs() {
        let p=root("partition");let a=p.join("apex");fs::create_dir(&a).unwrap();
        fs::write(a.join("com.test.capex"),b"metadata lookup does not open archive").unwrap();
        let roots=[(a,PathBuf::from("/system_ext/apex"))];
        assert_eq!(factory_partition(&roots,"com.test").unwrap(),"system_ext");
        assert!(factory_partition(&roots,"missing").is_err());
        assert!(!p.join("factory-apexes").exists());
        assert!(logical_partition(Path::new("/unexpected/system/apex/x.apex")).is_err());
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn normal_classpath_rejected() {
        assert!(check_explicit_modules(["{CLASSPATH}"].into_iter(),[].into_iter()).is_err());
    }
    #[test]
    fn tenant_classpath_rejected() {
        assert!(check_explicit_modules(["com.android.art"].into_iter(),["{CLASSPATH}"].into_iter()).is_err());
        assert!(check_explicit_modules(["com.android.art"].into_iter(),["com.android.i18n"].into_iter()).is_ok());
    }

}

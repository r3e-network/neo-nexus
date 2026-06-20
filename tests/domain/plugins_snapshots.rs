use super::*;

fn local_snapshot(
    id: &str,
    node_type: NodeType,
    source_path: impl AsRef<Path>,
    expected_sha256: &str,
) -> FastSyncSnapshot {
    FastSyncSnapshot {
        id: id.to_string(),
        label: format!("{id} snapshot"),
        network: Network::Testnet,
        node_type,
        source_path: source_path.as_ref().to_path_buf(),
        source_url: None,
        download_file_name: None,
        download_max_bytes: FastSyncSnapshotManager::DEFAULT_DOWNLOAD_MAX_BYTES,
        expected_sha256: expected_sha256.to_string(),
        cached_path: None,
        verified_sha256: None,
        verified_at_unix: None,
        bytes: None,
    }
}

fn write_tar_snapshot(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut builder = tar::Builder::new(file);
    for (entry_path, bytes) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, *entry_path, Cursor::new(*bytes))
            .unwrap();
    }
    builder.finish().unwrap();
}

fn write_tar_gzip_snapshot(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (entry_path, bytes) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, *entry_path, Cursor::new(*bytes))
            .unwrap();
    }
    builder.finish().unwrap();
}

fn write_zip_snapshot(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (entry_path, bytes) in entries {
        zip.start_file(*entry_path, options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

#[path = "plugins_snapshots/plugin_packages.rs"]
mod plugin_packages;
#[path = "plugins_snapshots/snapshot_archives.rs"]
mod snapshot_archives;
#[path = "plugins_snapshots/snapshot_catalogs.rs"]
mod snapshot_catalogs;
#[path = "plugins_snapshots/snapshot_downloads.rs"]
mod snapshot_downloads;
#[path = "plugins_snapshots/snapshot_repository.rs"]
mod snapshot_repository;

// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Adapt the upstream Amcache writer one record at a time; never buffer a whole hive/report.
use crate::output_schema;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct JsonOutput<'a> {
    out: &'a mut dyn Write,
    pretty: bool,
    pending: Vec<u8>,
    group: usize,
    first: bool,
}

const GROUPS: [&str; 4] = ["Summary", "Findings", "Files", "Devices"];

impl<'a> JsonOutput<'a> {
    pub fn new(out: &'a mut dyn Write, pretty: bool) -> Self {
        Self {
            out,
            pretty,
            pending: Vec::new(),
            group: 0,
            first: true,
        }
    }

    fn record(&mut self) -> io::Result<()> {
        if self.pending.iter().all(u8::is_ascii_whitespace) {
            self.pending.clear();
            return Ok(());
        }
        let mut value: Value = serde_json::from_slice(&self.pending)?;
        self.pending.clear();
        output_schema::normalize(&mut value)?;
        if !self.pretty {
            serde_json::to_writer(&mut self.out, &value)?;
            return self.out.write_all(b"\n");
        }
        let fields = value
            .as_object_mut()
            .ok_or_else(|| io::Error::other("Amcache record is not an object"))?;
        let kind = fields
            .remove("RecordType")
            .ok_or_else(|| io::Error::other("Amcache record has no RecordType"))?;
        let group = match kind.as_str() {
            Some("summary") => 0,
            Some("finding") => 1,
            Some("file") => 2,
            Some("device") => 3,
            _ => return Err(io::Error::other("Unknown Amcache record type")),
        };
        if group == 0 {
            if self.group != 0 {
                return Err(io::Error::other("Repeated Amcache summary"));
            }
            let mut common = serde_json::Map::new();
            for key in ["OutputSchemaVersion", "Tool", "Source", "AmcacheSchema"] {
                common.insert(
                    key.into(),
                    fields
                        .remove(key)
                        .ok_or_else(|| io::Error::other(format!("Amcache summary has no {key}")))?,
                );
            }
            self.out.write_all(b"{\n")?;
            for (key, value) in common {
                serde_json::to_writer(&mut self.out, &key)?;
                self.out.write_all(b": ")?;
                serde_json::to_writer_pretty(&mut self.out, &value)?;
                self.out.write_all(b",\n")?;
            }
            self.out.write_all(b"\"Summary\": ")?;
            serde_json::to_writer_pretty(&mut self.out, fields)?;
            self.out.write_all(b",\n\"Findings\": [\n")?;
            self.group = 1;
            return Ok(());
        }
        if self.group == 0 || group < self.group {
            return Err(io::Error::other("Amcache records are out of order"));
        }
        // Remove the envelope's duplicate common fields from each array member.
        for key in ["OutputSchemaVersion", "Tool", "Source", "AmcacheSchema"] {
            fields.remove(key);
        }
        self.advance(group)?;
        if !self.first {
            self.out.write_all(b",\n")?;
        }
        serde_json::to_writer_pretty(&mut self.out, fields)?;
        self.first = false;
        Ok(())
    }

    fn advance(&mut self, group: usize) -> io::Result<()> {
        while self.group < group {
            self.out.write_all(b"\n],\n")?;
            self.group += 1;
            serde_json::to_writer(&mut self.out, GROUPS[self.group])?;
            self.out.write_all(b": [\n")?;
            self.first = true;
        }
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        if !self.pending.is_empty() {
            self.record()?;
        }
        if self.pretty {
            if self.group == 0 {
                return Err(io::Error::other("Missing Amcache summary"));
            }
            self.advance(3)?;
            self.out.write_all(b"\n]\n}\n")?;
        }
        self.out.flush()
    }
}

impl Write for JsonOutput<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for fragment in bytes.split_inclusive(|&b| b == b'\n') {
            self.pending.extend_from_slice(fragment);
            if fragment.ends_with(b"\n") {
                self.record()?;
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

/// Rewrite only generated Amcache output. Keep row order/types and normalize record fields
/// embedded in JSON cells, including their evidence-provided registry name *values*.
pub fn csv(path: &Path) -> io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temporary = path.with_extension(format!(
        "csv-schema-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> io::Result<()> {
        let mut input = csv::Reader::from_reader(File::open(path)?);
        let headers = output_schema::headers(input.headers()?.iter())?;
        let embedded: Vec<_> = headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                matches!(h.as_str(), "RegistryValuesJSON" | "SubjectsJSON").then_some(i)
            })
            .collect();
        let mut out = csv::Writer::from_writer(BufWriter::new(file));
        out.write_record(&headers)?;
        for record in input.records() {
            let record = record?;
            let mut row: Vec<String> = record.iter().map(str::to_owned).collect();
            for &i in &embedded {
                if !row[i].is_empty() {
                    let mut value: Value = serde_json::from_str(&row[i])?;
                    output_schema::normalize(&mut value)?;
                    row[i] = serde_json::to_string(&value)?;
                }
            }
            out.write_record(&row)?;
        }
        out.flush()?;
        drop(out);
        drop(input);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn json_formats_preserve_report_shape_and_values_including_empty_groups() {
        let records = [
            json!({"output_schema_version":1,"tool":{"name":"tool","version":"1"},"source":"hive","amcache_schema":"modern","record_type":"summary","file_count":1}),
            json!({"output_schema_version":1,"tool":{},"source":"hive","amcache_schema":"modern","record_type":"file","name":"some_file.exe","registry_values":[{"name":"Some_Value","registry_type":"REG_SZ"}]}),
        ];
        for pretty in [false, true] {
            let mut out = Vec::new();
            let mut writer = JsonOutput::new(&mut out, pretty);
            for record in &records {
                let bytes = serde_json::to_vec(record).unwrap();
                // Exercise arbitrary writer boundaries, even inside UTF-8/escaped strings.
                for part in bytes.chunks(3) {
                    writer.write_all(part).unwrap();
                }
                writer.write_all(b"\n").unwrap();
            }
            writer.finish().unwrap();
            if pretty {
                let v: Value = serde_json::from_slice(&out).unwrap();
                assert_eq!(v["Summary"]["FileCount"], 1);
                assert_eq!(v["Findings"], json!([]));
                assert_eq!(v["Devices"], json!([]));
                assert_eq!(v["Files"][0]["RegistryValues"][0]["Name"], "Some_Value");
            } else {
                let v: Vec<Value> = String::from_utf8(out)
                    .unwrap()
                    .lines()
                    .map(|s| serde_json::from_str(s).unwrap())
                    .collect();
                assert_eq!(v[1]["RecordType"], "file");
                assert_eq!(v[1]["Name"], "some_file.exe");
            }
        }
    }
    #[test]
    fn csv_embedded_schema_changes_but_evidence_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("files.csv");
        let mut w = csv::Writer::from_path(&path).unwrap();
        w.write_record(["full_path", "registry_values_json"])
            .unwrap();
        w.write_record([
            "C:\\odd_name,\"file\"\n.exe",
            r#"[{"name":"lower_case","raw_data_hex":"00"}]"#,
        ])
        .unwrap();
        w.flush().unwrap();
        drop(w);
        csv(&path).unwrap();
        let mut r = csv::Reader::from_path(path).unwrap();
        assert_eq!(
            r.headers().unwrap(),
            &csv::StringRecord::from(vec!["FullPath", "RegistryValuesJSON"])
        );
        let row = r.records().next().unwrap().unwrap();
        assert_eq!(&row[0], "C:\\odd_name,\"file\"\n.exe");
        let value: Value = serde_json::from_str(&row[1]).unwrap();
        assert_eq!(value[0]["Name"], "lower_case");
        assert_eq!(value[0]["RawDataHex"], "00");
    }
}

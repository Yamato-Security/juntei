# Amcache

## What Amcache is

Amcache is a Windows application-compatibility inventory stored in a registry hive. Modern systems
record executable files under `InventoryApplicationFile` and Plug-and-Play devices under
`InventoryDevicePnp`; older Windows versions use a legacy `Root\File` layout.

For executable investigation, Amcache is valuable because it can preserve a full path, SHA-1,
file size, PE metadata, version information, publisher text, link date, and registry timestamps
after the original file has disappeared. Device records can identify attached hardware, drivers,
installation dates, and identifiers.

<span class="evidence-strength">Best conclusion:</span> Windows inventoried the recorded file or
device. An executable record is **evidence of presence, not proof of execution**.

## Location and collection

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Collect the hive from the affected installation. `parse-registry amcache` currently parses the primary
hive but does **not** replay adjacent Amcache transaction logs, so records present only in those
logs may be missing from a live or inconsistent collection. Note that limitation in the report.

## Analyst cautions

- An Amcache executable entry does not prove that a user launched the file.
- `publisher` is PE/version-resource metadata recorded by Windows. It does not prove the file had
  a valid Authenticode signature at collection time.
- Hashes may be absent and legacy/modern schemas do not expose identical values.
- `link_date` comes from the PE header and can be forged, copied, or produced by reproducible builds.
- A key last-write time describes the registry record's update, not necessarily download, creation,
  first execution, or last execution.
- Paths can refer to removable or no-longer-present volumes.

## Parse with Juntei

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.record_type == "file" and .sha1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV output contains summary, findings, files, and devices tables. JSONL records use
`record_type` values `summary`, `finding`, `file`, and `device`.

## File output fields

| Field | How an analyst should use it |
|---|---|
| `key_name`, `source_key_path` | Identify the exact source registry record and schema branch. Preserve for reproducibility. |
| `key_last_written_utc`, `key_last_written_filetime` | Registry record update time, in readable and raw form. Do not label it an execution time. |
| `legacy_volume_key_last_written_utc`, `legacy_volume_key_last_written_filetime` | Parent volume-key time on legacy schemas; contextual, not a file action time. |
| `file_id` | Amcache's recorded file identifier. Retained exactly; do not assume it is always a SHA-1. |
| `sha1`, `sha256` | Strong pivots into Defender, threat intelligence, EDR, and file repositories. Null means not recorded. |
| `name`, `original_file_name`, `description` | Recorded PE/version names. Compare with the actual path to detect masquerading; metadata is attacker-controlled. |
| `full_path` | Most useful location pivot. Normalize case carefully but retain the original evidence string. |
| `long_path_hash` | Windows' path-related identifier; useful for matching records, not a content hash. |
| `program_id` | Links the file to an application inventory record where available. |
| `publisher` | Recorded publisher text. Treat as metadata, not signature validation. |
| `version`, `bin_file_version`, `product_name`, `product_version`, `bin_product_version` | Version-resource context. Differences or implausible values can support masquerading analysis. |
| `binary_type` | Recorded architecture/type context; compare with PE analysis when the file is recovered. |
| `link_date` | PE compile/link timestamp as recorded. Useful only with skepticism and corroboration. |
| `size` | Recorded size; compare with `$MFT`, Defender content, or a recovered file. |
| `language` | PE language identifier; a weak contextual clue, not attribution. |
| `is_pe_file`, `is_os_component` | Windows' recorded classification flags. A false value does not prove harmlessness. |
| `usn` | NTFS update sequence value recorded by Amcache; can help pivot to surviving USN records. |
| `appx_package_full_name`, `appx_package_relative_id` | Associates packaged applications/components with the file. |
| `registry_values` | Every original registry value with type, decoded value, and raw hex. Use when a typed field is absent or for future/legacy fields. |

## Device output fields

| Field group | Fields and interpretation |
|---|---|
| Source | `key_name`, `source_key_path`, `key_last_written_utc`, `key_last_written_filetime`: exact device record and registry update time. |
| Identity | `description`, `bus_description`, `manufacturer`, `model`: human-facing device descriptions; vendor strings can be generic. |
| Hardware identifiers | `hwid`, `comp_id`, `class`, `class_guid`, `container_id`, `device_interface_classes`: strongest device-family and instance pivots. |
| State | `device_state`, `problem_code`, `parent_id`, `matching_id`, `enumerator`: Windows enumeration and relationship context. |
| Driver | `driver_id`, `driver_name`, `driver_package_strong_name`, `driver_ver_date`, `driver_ver_version`, `provider`, `service`, `inf`: driver package and service associations; correlate with installed files and services. |
| Installation | `install_date`, `first_install_date`, `install_state`, `extended_infs`: recorded install lifecycle. A missing date is not proof the device was never attached. |
| Filters and stack | `lower_class_filters`, `upper_class_filters`, `lower_filters`, `upper_filters`, `stack_id`, `location_paths`: advanced driver-stack context; unexpected filter drivers deserve review. |
| Raw preservation | `registry_values`: complete typed/raw source values for validation and unsupported fields. |

## Investigation strategy

1. Pivot on `full_path`, `sha1`, `name`, and `usn`.
2. Look for matching Prefetch to support execution.
3. Use `$MFT` and USN Journal to establish file creation, rename, deletion, and size.
4. Compare publisher/version metadata with an independently recovered PE and signature validation.
5. Check Defender quarantine and event logs for detection or removal.

## Further Details

- [Juntei's ys-amcache-parser output schema](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Eric Zimmerman's AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache plugin](https://github.com/mkorman90/regipy)

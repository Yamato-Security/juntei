# USB Devices

## What the USB history is

Windows records every device it installs. For USB storage, other USB devices and portable
devices (phones, cameras, media players), the `SYSTEM` hive keeps a key per device with its name,
serial number and the times Windows installed it, last saw it arrive and last saw it removed.
`MountedDevices` records the volume GUIDs and drive letters given to their volumes. The
`SOFTWARE` hive keeps the names Explorer gave portable devices and volumes, and ReadyBoost's
record of volume serial numbers. Each user's `NTUSER.DAT` records the volumes that user had
mounted.

`parse-registry usb` joins them into one record per device and volume: which device it was, when
it was installed and connected, which drive letter and volume it had, and which users had that
volume mounted.

<span class="evidence-strength">Best conclusion:</span> a device with this vendor, product and
serial number was connected to this system, first and last at these times; its volume had this
drive letter; these users had it mounted. Registry records don't show which files were copied.

## Location

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Device property | Field | Meaning (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: when its driver was last installed |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: when the device was first installed |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: when it was last connected (Windows 8 and later) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: when it was last removed (Windows 8 and later) |

Windows 8 and later keep each time as the default value of the `0064` … `0067` key. Windows 7
keeps it in `00000064\00000000\Data`. Juntei reads both.

Many DFIR references label `0064` as the first install and `0065` as the install. Juntei follows
Microsoft's `devpkey.h` definitions above, and names the columns after what each time records.

Give `-f` or `-d` the `SYSTEM` hive. The system's `SOFTWARE` hive and its users' `NTUSER.DAT`
hives are found next to it, as a collection keeps them (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), or in the same directory and one level down when the hives were
exported together. Collect all of them with their transaction logs.

## How the records are joined

- **Devices:** one record per device key: `usbstor`, `usb` and `wpdbusenum` (`DeviceClass`). A
  USB storage device takes its vendor and product IDs from the USB device with the same
  container ID (else the same serial number).
- **Volumes:** a USB storage device's volumes are the `\??\Volume{GUID}` values of
  `MountedDevices` whose data is a path that names the device. A device with two volumes has two
  records. Its drive letters are the `\DosDevices\X:` values with the same data.
- **Names and serial numbers:** volume names come from `Windows Portable Devices` and from the
  portable devices that name the same device. Volume labels and serial numbers come from
  `EMDMgmt` keys that name it.
- **Users:** the users whose `MountPoints2` has the volume's GUID, with that key's last-write time.
- **Volumes without a device key** (`volume`): a `MountedDevices` volume no device key names (an
  internal disk's, or a removed device's whose `Enum` key is gone), a drive letter no volume GUID
  value shares data with, and a volume only users' `MountPoints2` have.
- **Devices only `SOFTWARE` names** (`portable_device`, `readyboost`).

`SourceKeys` lists every key and value a record was joined from, with its hive.

## Analyst cautions

- **Drive letters are reused.** `MountedDevices` keeps the last device given each letter. A
  letter is shown only on the device whose data is still that letter's.
- **Serial numbers can be made up.** When the second character of an instance is `&`, Windows
  made it up because the device reports no serial number; `Serial` is empty then. Such an
  instance can differ on another computer.
- **USB storage instances end with the logical unit** (`&0`). `Serial` leaves it out; `Instance`
  keeps it.
- **Times differ by Windows version.** Windows 7 has only the install times. `LastArrivalUTC` and
  `LastRemovalUTC` are the last connection only, not every connection. Event logs
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) and
  `setupapi.dev.log` give more.
- **Control sets:** by default the control set Windows used last is read. `--all-control-sets`
  reads every control set; an older one can still list a device the current one no longer has.
- **A key's last-write time is not a connection time.** Windows rewrites device keys for many
  reasons.
- **Removed devices leave traces.** A volume record with a vendor and product but no device key
  is a device whose `Enum` key was removed (by an uninstall or by cleaning tools) whose volume
  `MountedDevices` still has.
- **ReadyBoost serial numbers** are the volume's serial number (`VolumeSerials`, as `dir` shows
  it), which LNK files and Jump Lists also record.

## Parse with Juntei

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.device_class == "usbstor") | {vendor, product, serial, last_arrival, drive_letters, users}'
```

The summary lists the USB storage devices that arrived last (newest first), with their drive
letters, and how many volumes belong to USB storage devices without a device key.

## Output fields

| Field | Interpretation |
|---|---|
| `Host` | The computer's name, from `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum`, or, without a device key, `volume`, `portable_device` or `readyboost`. |
| `ControlSet` | The control set of the device key. |
| `Vendor`, `Product`, `Revision` | USB storage: from the device's name (`Disk&Ven_…&Prod_…&Rev_…`), also for a device a volume or `SOFTWARE` key names. |
| `Vid`, `Pid` | The USB vendor and product IDs. |
| `Serial`, `Instance` | The serial number, and the instance key's name as written. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerId` | The device key's values, as written (`DeviceDesc` is often `@<inf>,%…%;<text>`). A portable device's `FriendlyName` is usually its volume's label or drive. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | The device property times (see above). |
| `DriveLetters` | The drive letters whose `MountedDevices` data is the volume's. |
| `VolumeGuid`, `VolumeData` | The volume's GUID, and what its `MountedDevices` data says it is on: a device path, an MBR disk signature and partition offset, or a GPT partition. |
| `VolumeNames`, `VolumeSerials` | Names Explorer gave the device or its volume, ReadyBoost labels, and volume serial numbers (`1234-ABCD`). |
| `Users` | The users whose `MountPoints2` has the volume, with that key's last write (JSON: `username`, `mount_points2_last_written`, `source_path`). |
| `FoundIn`, `SourceKeys` | Where it was found (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`), and every key and value it was joined from. |
| `KeyPath`, `KeyLastWrittenUTC` | The device's instance key and its last write. |
| `SourcePath`, `SoftwarePath` | The `SYSTEM` hive, and the `SOFTWARE` hive when it gave something. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context of the hives the record was joined from. |
| `Problems` | Keys that couldn't be read, and device properties that aren't FILETIMEs (with their bytes). |

## Investigation strategy

1. Sort USB storage devices by `LastArrivalUTC` (else `DriverLastInstalledUTC`), and set aside the
   computer's own readers and hubs.
2. Note each device's vendor, product, serial number, volume GUID, drive letter and volume serial
   number, and look for them in LNK files, Jump Lists, ShellBags and the `$MFT` (paths on that
   drive letter or volume).
3. Check `Users` for who had the volume mounted, and when (the `MountPoints2` key's last write).
4. Look for `volume` records that name a USB storage device: devices whose keys were removed.
5. Correlate with the event logs for every connection, not only the last.
6. Use `parse-registry read --recover-deleted` for deleted device keys.

## Further Details

- [Microsoft: Device properties (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Supporting mount manager requests](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (as Wine's headers have it)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): the device property numbers
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) and
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

User joins use each hive's source identity, so exported hives with the same profile name remain separate. `SourceKeys` retains the contributing user-hive paths. Companion-user discovery skips symbolic links, consistent with recursive hive discovery. VID/PID correlation requires a matching usable container ID; without one, serial matching must yield unambiguous IDs. Conflicting candidates remain unknown with a problem.

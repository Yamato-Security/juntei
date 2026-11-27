# Copyright (C) 2026 Shirofune Security Co., Ltd.
#
# Unless explicitly acquired and licensed from Licensor under another license, the contents of this
# file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
# allowed by the RPL, and You may not copy or use this file in either source code or executable
# form, except in compliance with the terms and conditions of the RPL.
#
# All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
# OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
# INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
# PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
# and limitations under the RPL.

"""Make sample.raw.gz: a raw 16 MiB disk (an MBR with one NTFS volume at sector 63) holding event
logs that are live, NTFS-compressed, deleted, or left in free space and in another file.

| Path on the volume | What it is |
| --- | --- |
| C/Windows/System32/winevt/Logs/ | a compressed folder (ntfs_attrib 0x800) |
| ... Logs/Security.evtx | live: evtx/new-user-security.evtx |
| ... Logs/Microsoft-Windows-TaskScheduler%4Operational.evtx | live: ../persistence/task-creation.evtx |
| ... Logs/Microsoft-Windows-HelloForBusiness%4Operational.evtx | deleted, compressed |
| C/Logs/Microsoft-Windows-WorkFolders%4WHC.evtx | deleted, not compressed (an empty page) |
| C/Temp/blob.tmp | deleted: an older copy of the Security chunk (its first 3 records) at byte 1000, and Application_no_crc32.evtx's chunk |
| C/TempPacked/packed.tmp | deleted, compressed: the TaskScheduler chunk with record IDs + 2000 at byte 4096 |
| C/pagefile.sys | the TaskScheduler chunk with record IDs + 1000 at byte 70000 |

Needs mkntfs, ntfs-3g (setuid root or run as root) and setfattr. Run from the repository's root:

    python3 test_files/carve_evtx/make_sample.py
"""

import gzip, hashlib, os, shutil, struct, subprocess, tempfile, zlib

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
HERE = os.path.join(REPO, "test_files", "carve_evtx")
OUT = os.path.join(HERE, "sample.raw.gz")
SECTOR = 512
MIB = 1 << 20
DISK = 16 * MIB
START = 63
VOLUME = DISK - START * SECTOR - MIB
CHUNK = 0x10000


def run(*args):
    subprocess.run(args, check=True, stdout=subprocess.DEVNULL)


def noise(n, seed):
    out = bytearray()
    counter = 0
    while len(out) < n:
        out += hashlib.sha256(f"{seed}:{counter}".encode()).digest()
        counter += 1
    return bytes(out[:n])


def read(name):
    with open(name, "rb") as f:
        return f.read()


def first_chunk(log):
    return bytearray(log[0x1000:0x1000 + CHUNK])


def records(chunk):
    last, free = struct.unpack_from("<II", chunk, 0x2C)
    offset, out = 0x200, []
    while offset <= last and offset < free:
        size = struct.unpack_from("<I", chunk, offset + 4)[0]
        out.append((offset, size))
        offset += size
    return out


def fix(chunk):
    free = struct.unpack_from("<I", chunk, 0x30)[0]
    struct.pack_into("<I", chunk, 0x34, zlib.crc32(bytes(chunk[0x200:free])))
    struct.pack_into("<I", chunk, 0x7C, zlib.crc32(bytes(chunk[:0x78]) + bytes(chunk[0x80:0x200])))
    return bytes(chunk)


def shrink(chunk, keep):
    """An older copy of a chunk: only its first `keep` records were written then."""
    chunk = bytearray(chunk)
    found = records(chunk)
    last_offset, last_size = found[keep - 1]
    first_number = struct.unpack_from("<Q", chunk, 0x08)[0]
    struct.pack_into("<Q", chunk, 0x10, first_number + keep - 1)
    struct.pack_into("<Q", chunk, 0x20, struct.unpack_from("<Q", chunk, last_offset + 8)[0])
    struct.pack_into("<II", chunk, 0x2C, last_offset, last_offset + last_size)
    chunk[last_offset + last_size:] = bytes(CHUNK - last_offset - last_size)
    return fix(chunk)


def renumber(chunk, delta):
    """The same events with other record IDs: different records for the deduplication."""
    chunk = bytearray(chunk)
    for offset, _ in records(chunk):
        rid = struct.unpack_from("<Q", chunk, offset + 8)[0]
        struct.pack_into("<Q", chunk, offset + 8, rid + delta)
    for field in (0x18, 0x20):
        struct.pack_into("<Q", chunk, field, struct.unpack_from("<Q", chunk, field)[0] + delta)
    return fix(chunk)


def write(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())


def compressed_dir(path):
    os.makedirs(path, exist_ok=True)
    subprocess.run(["setfattr", "-h", "-v", "0x00000800", "-n", "system.ntfs_attrib_be", path], check=True)


def fill(mount):
    c = os.path.join(mount, "C")
    security = read(os.path.join(HERE, "evtx", "new-user-security.evtx"))
    task = read(os.path.join(REPO, "test_files", "persistence", "task-creation.evtx"))
    hello = read(os.path.join(HERE, "evtx", "Microsoft-Windows-HelloForBusiness%4Operational.evtx"))
    folders = read(os.path.join(HERE, "evtx", "Microsoft-Windows-WorkFolders%4WHC.evtx"))
    application = read(os.path.join(HERE, "evtx", "Application_no_crc32.evtx"))

    logs = os.path.join(c, "Windows", "System32", "winevt", "Logs")
    compressed_dir(logs)
    write(os.path.join(logs, "Security.evtx"), security)
    write(os.path.join(logs, "Microsoft-Windows-TaskScheduler%4Operational.evtx"), task)
    deleted = [
        os.path.join(logs, "Microsoft-Windows-HelloForBusiness%4Operational.evtx"),
        os.path.join(c, "Logs", "Microsoft-Windows-WorkFolders%4WHC.evtx"),
        os.path.join(c, "Temp", "blob.tmp"),
        os.path.join(c, "TempPacked", "packed.tmp"),
    ]
    write(deleted[0], hello)
    write(deleted[1], folders)
    write(deleted[2], noise(1000, "blob") + shrink(first_chunk(security), 3) + noise(777, "gap")
          + first_chunk(application) + noise(3000, "tail"))
    compressed_dir(os.path.dirname(deleted[3]))
    write(deleted[3], bytes(0x1000) + renumber(first_chunk(task), 2000) + bytes(2 * CHUNK))
    write(os.path.join(c, "pagefile.sys"),
          noise(70000, "page") + renumber(first_chunk(task), 1000) + noise(5000, "page-tail"))
    for path in deleted:
        os.remove(path)


def main():
    work = tempfile.mkdtemp()
    try:
        volume = os.path.join(work, "volume.img")
        with open(volume, "wb") as f:
            f.truncate(VOLUME)
        run("mkntfs", "-F", "-Q", "-q", "-s", str(SECTOR), "-c", "4096", "-p", str(START),
            "-H", "255", "-S", "63", "-L", "CARVE", volume)
        mount = os.path.join(work, "mnt")
        os.makedirs(mount)
        run("ntfs-3g", "-o", "compression,big_writes", volume, mount)
        try:
            fill(mount)
        finally:
            run("fusermount3", "-u", mount)
        vol = read(volume)
        disk = bytearray(DISK)
        mbr = bytearray(SECTOR)
        struct.pack_into("<I", mbr, 440, 0x45565458)
        entry = 446
        mbr[entry + 4] = 0x07
        struct.pack_into("<II", mbr, entry + 8, START, VOLUME // SECTOR)
        mbr[510:512] = b"\x55\xaa"
        disk[0:SECTOR] = mbr
        disk[START * SECTOR:START * SECTOR + VOLUME] = vol
        with gzip.GzipFile(OUT, "wb", mtime=0) as f:
            f.write(bytes(disk))
        print(f"{OUT}: {os.path.getsize(OUT)} bytes, disk SHA-256 {hashlib.sha256(disk).hexdigest()}")
    finally:
        shutil.rmtree(work)


if __name__ == "__main__":
    main()

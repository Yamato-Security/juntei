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

"""Make sample.vhdx.gz: a small dynamic VHDX laid out like the VHDX files of triage collectors.

An MBR with one NTFS partition at sector 63 whose boot sector has its jump instruction and 55 AA
signature zeroed, as .NET DiscUtils writes them, holding a `C` folder with files from
Juntei's own test data and files made to exercise the reader: a fragmented file, a sparse one, a
compressed one, a resident one, an empty one and a non-ASCII name.

Needs mkntfs and ntfs-3g (ntfs-3g mounts the volume to fill it; it must be setuid root or run as
root). Run from the repository's root:

    python3 test_files/vhdx/make_sample.py

It prints the SHA-256 of every file, as the tests expect them.
"""

import gzip, hashlib, os, shutil, struct, subprocess, sys, tempfile, uuid, zlib

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(REPO, "test_files", "vhdx", "sample.vhdx.gz")
SECTOR = 512
MIB = 1 << 20
DISK = 32 * MIB
START = 63                       # the partition's first sector, as DiscUtils puts it
VOLUME = DISK - START * SECTOR - MIB
BLOCK = MIB                      # the VHDX block size


def run(*args):
    subprocess.run(args, check=True, stdout=subprocess.DEVNULL)


def pattern(n, seed):
    """`n` bytes no two blocks of which repeat."""
    out = bytearray()
    counter = 0
    while len(out) < n:
        out += hashlib.sha256(f"{seed}:{counter}".encode()).digest()
        counter += 1
    return bytes(out[:n])


def fill(mount):
    c = os.path.join(mount, "C")
    files = {
        "Windows/Prefetch/CMD.EXE-6D6290C5.pf": "test_files/prefetch/CMD.EXE-6D6290C5.pf",
        "Windows/System32/winevt/Logs/Microsoft-Windows-TaskScheduler%4Operational.evtx":
            "test_files/persistence/task-creation.evtx",
        "Windows/System32/LogFiles/Sum/Current.mdb": "test_files/ual/server2019/Current.mdb",
        "Windows/System32/LogFiles/Sum/SystemIdentity.mdb": "test_files/ual/server2019/SystemIdentity.mdb",
    }
    for target, source in files.items():
        path = os.path.join(c, target)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        shutil.copyfile(os.path.join(REPO, source), path)
    data = os.path.join(c, "Users/sample/Data")
    os.makedirs(data)
    # resident, empty and non-ASCII
    with open(os.path.join(data, "small.txt"), "wb") as f:
        f.write(b"resident content\n")
    open(os.path.join(data, "empty.bin"), "wb").close()
    with open(os.path.join(data, "日本語のファイル.txt"), "wb") as f:
        f.write("名前がUnicodeのファイル\n".encode())
    # fragmented: written in turns with another file, so its clusters alternate with the other's
    with open(os.path.join(data, "fragmented.bin"), "wb") as a, open(os.path.join(data, "spacer.bin"), "wb") as b:
        for i in range(8):
            a.write(pattern(16 * 1024 + 123, f"fragmented{i}"))
            a.flush(); os.fsync(a.fileno())
            b.write(pattern(16 * 1024, f"spacer{i}"))
            b.flush(); os.fsync(b.fileno())
    # sparse: data only at the start and the end of 2 MiB
    with open(os.path.join(data, "sparse.bin"), "wb") as f:
        f.write(pattern(4096, "sparse-head"))
        f.seek(2 * MIB - 4096)
        f.write(pattern(4096, "sparse-tail"))
    # compressed: a directory with the compressed attribute (ntfs-3g's ntfs_attrib)
    packed = os.path.join(data, "compressed")
    os.makedirs(packed)
    subprocess.run(["setfattr", "-h", "-v", "0x00000800", "-n", "system.ntfs_attrib_be", packed], check=True)
    with open(os.path.join(packed, "text.log"), "wb") as f:
        f.write(b"".join(f"line {i:06} of a compressible log\n".encode() for i in range(20000)))


def crc32c(data):
    table = getattr(crc32c, "table", None)
    if table is None:
        table = []
        for i in range(256):
            c = i
            for _ in range(8):
                c = (c >> 1) ^ 0x82F63B78 if c & 1 else c >> 1
            table.append(c)
        crc32c.table = table
    c = 0xFFFFFFFF
    for x in data:
        c = table[(c ^ x) & 0xFF] ^ (c >> 8)
    return c ^ 0xFFFFFFFF


def vhdx(raw):
    """A dynamic VHDX of `raw` (1 MiB blocks; blocks of zeros are left out)."""
    assert len(raw) % BLOCK == 0
    blocks = len(raw) // BLOCK
    out = bytearray(4 * MIB)
    out[0:8] = b"vhdxfile"
    out[8:8 + 30] = "Juntei test sample".encode("utf-16-le")[:30]
    file_write, data_write = uuid.UUID(int=1), uuid.UUID(int=2)
    for seq, off in ((1, 0x10000), (2, 0x20000)):
        h = bytearray(4096)
        h[0:4] = b"head"
        struct.pack_into("<Q", h, 8, seq)
        h[16:32] = file_write.bytes_le
        h[32:48] = data_write.bytes_le
        struct.pack_into("<HHIQ", h, 64, 0, 1, MIB, MIB)
        struct.pack_into("<I", h, 4, crc32c(h))
        out[off:off + 4096] = h
    bat_guid = uuid.UUID("2dc27766-f623-4200-9d64-115e9bfd4a08")
    meta_guid = uuid.UUID("8b7ca206-4790-4b9a-b8fe-575f050f886e")
    for off in (0x30000, 0x40000):
        r = bytearray(0x10000)
        r[0:4] = b"regi"
        struct.pack_into("<I", r, 8, 2)
        r[16:32] = bat_guid.bytes_le
        struct.pack_into("<QII", r, 32, 3 * MIB, MIB, 1)
        r[48:64] = meta_guid.bytes_le
        struct.pack_into("<QII", r, 64, 2 * MIB, MIB, 1)
        struct.pack_into("<I", r, 4, crc32c(r))
        out[off:off + 0x10000] = r
    # metadata at 2 MiB: five items from 64 KiB into the region
    items = [
        ("caa16737-fa36-4d43-b3b6-33f0aa44e76b", struct.pack("<II", BLOCK, 0), 0b100),
        ("2fa54224-cd1b-4876-b211-5dbed83bf4b8", struct.pack("<Q", len(raw)), 0b110),
        ("beca12ab-b2e6-4523-93ef-c309e000c746", uuid.UUID(int=3).bytes_le, 0b110),
        ("8141bf1d-a96f-4709-ba47-f233a8faab5f", struct.pack("<I", SECTOR), 0b110),
        ("cda348c7-445d-4471-9cc9-e9885251c556", struct.pack("<I", 4096), 0b110),
    ]
    m = 2 * MIB
    out[m:m + 8] = b"metadata"
    struct.pack_into("<H", out, m + 10, len(items))
    at = 0x10000
    for i, (guid, value, flags) in enumerate(items):
        e = m + 32 + 32 * i
        out[e:e + 16] = uuid.UUID(guid).bytes_le
        struct.pack_into("<III", out, e + 16, at, len(value), flags)
        out[m + at:m + at + len(value)] = value
        at += 0x1000
    # BAT at 3 MiB, payload from 4 MiB
    for b in range(blocks):
        chunk = raw[b * BLOCK:(b + 1) * BLOCK]
        if chunk.count(0) == len(chunk):
            continue
        offset = len(out)
        out += chunk
        struct.pack_into("<Q", out, 3 * MIB + 8 * b, (offset // MIB) << 20 | 6)
    return bytes(out)


def main():
    work = tempfile.mkdtemp()
    try:
        volume = os.path.join(work, "volume.img")
        with open(volume, "wb") as f:
            f.truncate(VOLUME)
        run("mkntfs", "-F", "-Q", "-q", "-s", str(SECTOR), "-c", "4096", "-p", str(START),
            "-H", "255", "-S", "63", "-L", "TEST", volume)
        mount = os.path.join(work, "mnt")
        os.makedirs(mount)
        run("ntfs-3g", "-o", "compression,big_writes", volume, mount)
        try:
            fill(mount)
        finally:
            run("fusermount3", "-u", mount)
        with open(volume, "rb") as f:
            vol = bytearray(f.read())
        # DiscUtils leaves the jump instruction and the signature zeroed
        vol[0:3] = b"\0\0\0"
        vol[510:512] = b"\0\0"
        disk = bytearray(DISK)
        mbr = bytearray(SECTOR)
        struct.pack_into("<I", mbr, 440, 0x4A554E54)
        entry = 446
        mbr[entry] = 0
        mbr[entry + 4] = 0x07
        struct.pack_into("<II", mbr, entry + 8, START, VOLUME // SECTOR)
        mbr[510:512] = b"\x55\xaa"
        disk[0:SECTOR] = mbr
        disk[START * SECTOR:START * SECTOR + VOLUME] = vol
        image = vhdx(bytes(disk))
        with gzip.GzipFile(OUT, "wb", mtime=0) as f:
            f.write(image)
        print(f"{OUT}: VHDX {len(image)} bytes, gzipped {os.path.getsize(OUT)}")
        # the files' hashes, from the volume as ntfs-3g reads it
        run("ntfs-3g", "-o", "ro", volume, mount)
        try:
            for root, _, names in os.walk(mount):
                for name in sorted(names):
                    path = os.path.join(root, name)
                    with open(path, "rb") as f:
                        digest = hashlib.sha256(f.read()).hexdigest()
                    print(f"{os.path.relpath(path, mount)}\t{os.path.getsize(path)}\t{digest}")
        finally:
            run("fusermount3", "-u", mount)
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()

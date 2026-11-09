# E01 test image

`sample.E01` and `sample.E02` are one E01 image in two segments: the 32 MiB disk of
`../vhdx/sample.vhdx.gz` (an MBR with one NTFS volume, made by `../vhdx/make_sample.py`), followed
by 1,310,720 bytes of `random.Random(140).getrandbits(8)` from Python, which take the compressed
image past one 1 MiB segment.

It was written by libewf's `ewfacquire` 20260924, as an EnCase 6 image with best deflate
compression:

```bash
ewfacquire -u -t sample -f encase6 -c deflate:best -S 1MiB -C JUNTEI-TEST \
  -D "Juntei test image" -E Juntei -e Juntei -N "made from test_files/vhdx/sample.vhdx.gz" \
  -m fixed -M physical -b 64 -q sample-padded.raw
```

| File | SHA-256 |
| --- | --- |
| `sample.E01` | `4bc3c596b2689aeb026520529b9c871e3c5de74e20e745eba8b50528a217dbad` |
| `sample.E02` | `1ce785fda95ba32fa5b0d436ab02575a42d4999196f0f423555e90c1fb7c3d01` |

The media (34,865,152 bytes, MD5 `0797559f396818da2e71f0254064bcf4`, SHA-256
`3eca4b11ef99864745cf3f6a3153868d9cf659f7348c269ba51a65e91beefd4c`) reads the same with libewf
(pyewf 20260924) and with ys-e01-parser.


## EWF2 and bare-volume fixtures

`sample.Ex01` contains the same complete media as `sample.E01`/`sample.E02` in a
single EWF2 segment. `volume.Ex01` contains only its NTFS volume: start at the first
MBR partition's LBA and take the sector count in the NTFS boot sector (32,473,088 bytes).
They were independently written by libewf 20260924 and read back with pyewf, comparing
every decoded byte with the source. The images contain only the existing public test data.

```bash
ewfacquire -u -t sample -f encase7-v2 -c deflate:best -S 2GiB -C JUNTEI-TEST \
  -D "Public Juntei test image" -E Juntei -e Juntei \
  -N "Derived from Juntei public VHDX fixture" -m fixed -M physical -b 64 -q sample.raw
```

Use the same command with `-t volume` and `volume.raw` for the bare NTFS image.

| File | SHA-256 |
| --- | --- |
| `sample.Ex01` | `0fab04a490a1548db01ead6247a8b6db95d29c6d17fa094b9f82816d9cb11e85` |
| `volume.Ex01` | `d1457c80877fcf5677c8c82607364493acad269fc412352f232253d8938fa0d7` |

The bare volume's decoded SHA-256 is
`02cc22dc0e362552adde74d166562e934b1795ef19261c22d1b436d796a569ad`.

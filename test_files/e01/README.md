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

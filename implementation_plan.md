# Fix MP4 Extraction Corruption (moov atom not found)

The issue where FFmpeg fails with `moov atom not found` or `Invalid data found when processing input` is caused by a bug in the custom Rust `.pak` parser when extracting the MP4 payload.

## The Bug

When extracting a file from the `.pak`, the engine stores a **Payload Header** (a serialized `FPakEntry`) right before the actual file data. The `.pak` index tells us the `offset` of this header, not the data itself.

In the Rust code, the parser attempts to read the `Offset` from this payload header. However, in Unreal Engine 4's format, the `Offset` field is **only serialized in the index**, never in the payload header. Because of this mismatch:
1. The parser misaligns and fails to correctly skip the payload header (which is typically 45 bytes long).
2. It ends up reading the payload header as if it were part of the encrypted MP4 data.
3. The AES-256-ECB decryption is then applied to the data, but because the start of the data is shifted by 45 bytes, the 16-byte AES blocks are misaligned.
4. This results in the entire MP4 file being decrypted into complete garbage data, which FFmpeg cannot read.

## Proposed Changes

### `tauri-app/src-tauri/src/pak/parser.rs`

#### [MODIFY] `parser.rs`
1. Update `extract_file` to properly skip the payload header by adding its exact size instead of trying to read it conditionally.
2. The payload header size is calculated as:
   - `Size` (8) + `UncompressedSize` (8) + `CompressionMethod` (4) + `Hash` (20) = 40 bytes.
   - If compressed (`compression_method != 0`), read the `Count` (4) and skip the compression blocks (`Count * 16`).
   - Skip `bEncrypted` (1) and `CompressionBlockSize` (4) = 5 bytes.
3. Total base header size is 45 bytes (plus compression blocks if applicable).
4. Remove the faulty `offset_check` logic.

## Verification Plan
1. Re-compile the Tauri app.
2. The user will test exporting `M0362_Nvzhu.mp4` again.
3. The newly extracted `.mp4` file should start with a valid MP4 header (e.g., `ftyp`) and FFmpeg will successfully process it without the `moov atom not found` error.

## User Review Required
> [!IMPORTANT]
> The current MP4 files in the `data/output/` folder are corrupted and will need to be re-extracted. The tool will automatically overwrite them when processing again, but if extraction was skipped, we will need to delete the corrupt MP4s or force extraction. I will add a step in the code to ensure we force re-extract if the file is corrupted or just tell you to delete the `output` folder.

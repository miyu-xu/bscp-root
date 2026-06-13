#!/usr/bin/env python3
"""Create a GPT disk with AOSP super + GPP boot_a partitions."""
import struct, uuid, os, zlib

SECTOR = 512
GPT_ENTRY_SIZE = 128
GPT_NUM_ENTRIES = 128
ALIGN = 2048

# Read AOSP super
aosp_super_path = 'C:/workspace/bscp/bscp/out/dist/img/super_aosp_ext4.img'
super_size = os.path.getsize(aosp_super_path)
print(f'AOSP super size: {super_size} bytes ({super_size/1024/1024:.1f} MB)')

# Read GPP aggregate to get boot_a data and partition type GUIDs
gpp_path = 'C:/workspace/bscp/bscp/out/dist/img/aggregate.img'
with open(gpp_path, 'rb') as f:
    f.seek(512)
    header = f.read(92)
    entries_lba = struct.unpack_from('<Q', header, 72)[0]
    num_entries = struct.unpack_from('<I', header, 80)[0]
    entry_size = struct.unpack_from('<I', header, 84)[0]

    f.seek(entries_lba * SECTOR)
    gpp_partitions = {}
    for i in range(num_entries):
        entry = f.read(entry_size)
        name = entry[56:128].decode('utf-16-le').rstrip('\x00')
        if name:
            first_lba = struct.unpack_from('<Q', entry, 32)[0]
            last_lba = struct.unpack_from('<Q', entry, 40)[0]
            gpp_partitions[name] = {
                'type_guid': entry[0:16],
                'first_lba': first_lba,
                'last_lba': last_lba,
                'sectors': last_lba - first_lba + 1,
            }
            print(f'GPP partition: {name} ({gpp_partitions[name]["sectors"]} sectors)')

# Extract boot_a data
boot_info = gpp_partitions['boot_a']
boot_sectors = boot_info['sectors']
boot_offset = boot_info['first_lba'] * SECTOR
with open(gpp_path, 'rb') as gf:
    gf.seek(boot_offset)
    boot_data = gf.read(boot_sectors * SECTOR)
print(f'Read boot_a: {len(boot_data)} bytes')

# Layout
super_start = ALIGN
super_sectors = (super_size + SECTOR - 1) // SECTOR
super_end = super_start + super_sectors - 1

boot_a_start = ((super_end + 1 + ALIGN - 1) // ALIGN) * ALIGN
boot_a_sectors = boot_sectors
boot_a_end = boot_a_start + boot_a_sectors - 1

secondary_table_offset = boot_a_end + 1
# Align disk to 1MB
disk_sectors = ((secondary_table_offset + 32 + 1 + ALIGN - 1) // ALIGN) * ALIGN

print(f'\nNew disk layout:')
print(f'  super: LBA {super_start}-{super_end} ({super_sectors} sectors, {super_sectors*SECTOR/1024/1024:.1f} MB)')
print(f'  boot_a: LBA {boot_a_start}-{boot_a_end} ({boot_a_sectors} sectors, {boot_a_sectors*SECTOR/1024/1024:.1f} MB)')
print(f'  Disk: {disk_sectors} sectors ({disk_sectors*SECTOR/1024/1024:.1f} MB)')

# Build GPT entries
entries = bytearray(GPT_NUM_ENTRIES * GPT_ENTRY_SIZE)

# Super partition entry
super_type = gpp_partitions['super']['type_guid']
super_part_guid = uuid.uuid4()
super_name = 'super'.encode('utf-16-le')
super_entry = bytearray(GPT_ENTRY_SIZE)
super_entry[0:16] = super_type
super_entry[16:32] = super_part_guid.bytes_le
struct.pack_into('<Q', super_entry, 32, super_start)
struct.pack_into('<Q', super_entry, 40, super_end)
super_entry[56:56+len(super_name)] = super_name
entries[0:GPT_ENTRY_SIZE] = super_entry

# boot_a partition entry
boot_type = gpp_partitions['boot_a']['type_guid']
boot_part_guid = uuid.uuid4()
boot_name = 'boot_a'.encode('utf-16-le')
boot_entry = bytearray(GPT_ENTRY_SIZE)
boot_entry[0:16] = boot_type
boot_entry[16:32] = boot_part_guid.bytes_le
struct.pack_into('<Q', boot_entry, 32, boot_a_start)
struct.pack_into('<Q', boot_entry, 40, boot_a_end)
boot_entry[56:56+len(boot_name)] = boot_name
entries[GPT_ENTRY_SIZE:2*GPT_ENTRY_SIZE] = boot_entry

# CRC
entries_crc = zlib.crc32(entries) & 0xFFFFFFFF

# Build primary GPT header
disk_guid = uuid.uuid4()
primary_header = bytearray(92)
primary_header[0:8] = b'EFI PART'
struct.pack_into('<I', primary_header, 8, 0x00010000)  # Revision
struct.pack_into('<I', primary_header, 12, 92)  # Header size
# CRC at 16-19 (zeroed for calculation)
struct.pack_into('<Q', primary_header, 24, 1)  # My LBA
struct.pack_into('<Q', primary_header, 32, disk_sectors - 1)  # Backup LBA
struct.pack_into('<Q', primary_header, 40, ALIGN)  # First usable LBA
struct.pack_into('<Q', primary_header, 48, secondary_table_offset - 1)  # Last usable LBA
guid_bytes = disk_guid.bytes_le
struct.pack_into('<Q', primary_header, 56, int.from_bytes(guid_bytes[0:8], 'little'))
struct.pack_into('<Q', primary_header, 64, int.from_bytes(guid_bytes[8:16], 'little'))
struct.pack_into('<Q', primary_header, 72, 2)  # Partition entries LBA
struct.pack_into('<I', primary_header, 80, GPT_NUM_ENTRIES)
struct.pack_into('<I', primary_header, 84, GPT_ENTRY_SIZE)
struct.pack_into('<I', primary_header, 88, entries_crc)

# Calculate header CRC
hdr_for_crc = bytearray(primary_header)
struct.pack_into('<I', hdr_for_crc, 16, 0)
header_crc = zlib.crc32(hdr_for_crc[:92]) & 0xFFFFFFFF
struct.pack_into('<I', primary_header, 16, header_crc)

# Build backup GPT header
backup_header = bytearray(primary_header)
struct.pack_into('<Q', backup_header, 24, disk_sectors - 1)  # My LBA = last sector
struct.pack_into('<Q', backup_header, 32, 1)  # Backup LBA = primary location
struct.pack_into('<Q', backup_header, 72, secondary_table_offset)  # Partition entries LBA

bhdr_for_crc = bytearray(backup_header)
struct.pack_into('<I', bhdr_for_crc, 16, 0)
backup_hdr_crc = zlib.crc32(bhdr_for_crc[:92]) & 0xFFFFFFFF
struct.pack_into('<I', backup_header, 16, backup_hdr_crc)

# Write disk
output_path = 'C:/workspace/bscp/bscp/out/dist/img/aggregate_aosp.img'

with open(output_path, 'wb') as f:
    # Protective MBR at LBA 0
    mbr = bytearray(512)
    mbr[0x1BE + 4] = 0xEE
    mbr_size = min(disk_sectors - 1, 0xFFFFFFFF)
    struct.pack_into('<I', mbr, 0x1BE + 8, 1)
    struct.pack_into('<I', mbr, 0x1BE + 12, mbr_size)
    mbr[0x1FE] = 0x55
    mbr[0x1FF] = 0xAA
    f.write(mbr)

    # Primary GPT header at LBA 1 (92 bytes, pad to fill sector)
    f.write(primary_header)
    f.write(b'\x00' * (SECTOR - len(primary_header)))

    # Partition entries at LBA 2
    # f.tell() should already be at LBA 2 since we padded header to SECTOR size
    f.write(entries)

    # Pad to super_start
    current_sector = f.tell() // SECTOR
    pad_sectors = super_start - current_sector
    f.write(b'\x00' * (pad_sectors * SECTOR))

    # Write super partition
    print(f'Writing super at offset {f.tell()} ({f.tell()//SECTOR} sectors)...')
    with open(aosp_super_path, 'rb') as sf:
        while True:
            chunk = sf.read(8*1024*1024)
            if not chunk:
                break
            f.write(chunk)

    # Pad super to expected end
    current = f.tell()
    expected = (super_end + 1) * SECTOR
    if current < expected:
        f.write(b'\x00' * (expected - current))

    # Write boot_a
    print(f'Writing boot_a at offset {f.tell()} ({f.tell()//SECTOR} sectors)...')
    f.write(boot_data)

    # Pad boot_a to expected end
    current = f.tell()
    expected = (boot_a_end + 1) * SECTOR
    if current < expected:
        f.write(b'\x00' * (expected - current))

    # Write backup partition entries
    f.seek(secondary_table_offset * SECTOR)
    f.write(entries)

    # Write backup GPT header at last sector
    f.seek((disk_sectors - 1) * SECTOR)
    f.write(backup_header)

    # Ensure file is exactly disk_sectors size
    f.seek(disk_sectors * SECTOR - 1)
    f.write(b'\x00')

print(f'\nCreated: {output_path}')
actual_size = os.path.getsize(output_path)
print(f'Size: {actual_size} bytes = {actual_size/SECTOR:.0f} sectors ({actual_size/1024/1024:.1f} MB)')

# Verify
with open(output_path, 'rb') as f:
    f.seek(512)
    sig = f.read(8)
    print(f'GPT signature: {sig}')
    f.seek(2*SECTOR)
    entry = f.read(GPT_ENTRY_SIZE)
    name = entry[56:128].decode('utf-16-le').rstrip('\x00')
    print(f'Partition 0: {name}')
    entry2 = f.read(GPT_ENTRY_SIZE)
    name2 = entry2[56:128].decode('utf-16-le').rstrip('\x00')
    print(f'Partition 1: {name2}')

print('Done!')

#!/usr/bin/env python3
"""Extract logical partitions from an AOSP super image."""
import struct, sys, os

def parse_lp_metadata(data):
    """Parse LpMetadata from binary data."""
    # LpMetadataHeader
    magic = struct.unpack_from('<I', data, 0)[0]
    if magic != 0x616C4467:
        print(f'Bad LP magic: 0x{magic:08X}')
        return None

    major = struct.unpack_from('<H', data, 4)[0]
    minor = struct.unpack_from('<H', data, 6)[0]
    header_size = struct.unpack_from('<I', data, 8)[0]
    header_checksum = struct.unpack_from('<I', data, 12)[0]
    tables_size = struct.unpack_from('<I', data, 16)[0]
    tables_checksum = struct.unpack_from('<I', data, 20)[0]

    part_count = struct.unpack_from('<I', data, 24)[0]
    extent_count = struct.unpack_from('<I', data, 28)[0]
    group_count = struct.unpack_from('<I', data, 32)[0]
    block_devices_count = struct.unpack_from('<I', data, 36)[0]

    flags = struct.unpack_from('<I', data, 40)[0]

    print(f'LP metadata v{major}.{minor}: {part_count} partitions, {extent_count} extents, {group_count} groups')

    # Parse partitions
    offset = 44  # After fixed header
    partitions = {}
    for i in range(part_count):
        name = data[offset:offset+36].decode('utf-8').rstrip('\x00')
        attr = struct.unpack_from('<I', data, offset+36)[0]
        first_extent = struct.unpack_from('<I', data, offset+40)[0]
        num_extents = struct.unpack_from('<I', data, offset+44)[0]
        group_idx = struct.unpack_from('<I', data, offset+48)[0]
        part_flags = struct.unpack_from('<I', data, offset+52)[0]
        partitions[name] = {
            'name': name,
            'first_extent': first_extent,
            'num_extents': num_extents,
            'group_idx': group_idx,
        }
        offset += 56
        print(f'  Partition: {name} (extents {first_extent}-{first_extent+num_extents-1}, group {group_idx})')

    # Parse extents
    extents = []
    for i in range(extent_count):
        num_sectors = struct.unpack_from('<Q', data, offset)[0]
        target_type = struct.unpack_from('<I', data, offset+8)[0]
        target_data = struct.unpack_from('<Q', data, offset+12)[0]
        target_source = struct.unpack_from('<I', data, offset+20)[0]
        extents.append({
            'num_sectors': num_sectors,
            'target_type': target_type,
            'target_data': target_data,
            'target_source': target_source,
        })
        offset += 24

    for i, e in enumerate(extents):
        ttype = {0: 'linear', 1: 'zero'}.get(e['target_type'], f'unknown({e["target_type"]})')
        print(f'  Extent {i}: {e["num_sectors"]} sectors, type={ttype}, target_data={e["target_data"]}, target_source={e["target_source"]}')

    return partitions, extents


def main():
    super_path = sys.argv[1] if len(sys.argv) > 1 else r'C:\workspace\bscp\bscp\out\dist\img\super_aosp_ext4.img'
    out_dir = sys.argv[2] if len(sys.argv) > 2 else r'C:\workspace\bscp\bscp\out\dist\img'

    SECTOR = 512

    with open(super_path, 'rb') as f:
        # Try each metadata slot
        for slot in range(4):
            f.seek(4096 + slot * 65536)
            hdr = f.read(4096)
            magic = struct.unpack_from('<I', hdr, 0)[0]
            if magic == 0x616C4467:
                print(f'Found LP metadata at slot {slot}')
                partitions, extents = parse_lp_metadata(hdr)

                # Extract target partitions
                targets = ['odm_a', 'vendor_a']
                for part_name in targets:
                    if part_name not in partitions:
                        print(f'Partition {part_name} not found')
                        continue

                    p = partitions[part_name]
                    total_sectors = 0

                    # Calculate total size
                    for ei in range(p['first_extent'], p['first_extent'] + p['num_extents']):
                        total_sectors += extents[ei]['num_sectors']

                    print(f'\nExtracting {part_name}: {total_sectors} sectors = {total_sectors * SECTOR / 1024 / 1024:.1f} MB')

                    out_path = os.path.join(out_dir, f'{part_name}.img')
                    with open(out_path, 'wb') as out:
                        for ei in range(p['first_extent'], p['first_extent'] + p['num_extents']):
                            e = extents[ei]
                            if e['target_type'] == 0:  # linear
                                # target_data is the sector offset in the super image
                                super_offset = e['target_data'] * SECTOR
                                f.seek(super_offset)
                                out.write(f.read(e['num_sectors'] * SECTOR))
                            elif e['target_type'] == 1:  # zero
                                out.write(b'\x00' * e['num_sectors'] * SECTOR)

                    print(f'  Wrote {out_path} ({os.path.getsize(out_path)} bytes)')

                break
        else:
            print('ERROR: No valid LP metadata found')
            return

    print('\nDone!')

if __name__ == '__main__':
    main()

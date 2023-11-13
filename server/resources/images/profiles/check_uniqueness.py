# Grab all png files in the script folder, and check if there are any duplicates.

import sys
from glob import glob
from hashlib import md5

current_path = sys.path[0]
profile_files = glob(f'{current_path}/*.png')

print('File count:', len(profile_files))

m = md5()
hashes = dict()
for filename in profile_files:
    with open(filename, 'rb') as f:
        m.update(f.read())
    hash = m.digest()

    assert hash not in hashes, f'Collision occurred: {hashes[hash]}, {filename}'
    hashes[hash] = filename

print('Check successful, no duplicate files found.')
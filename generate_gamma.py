import argparse

parser = argparse.ArgumentParser(description="Generate an 8 bit gamma remap table for rust code")
parser.add_argument('--gamma', dest='gamma', action='store', required=True );

args = parser.parse_args()
gamma = float(args.gamma);

print("const gamma : [u8; 256 ] = [", end='')

for number in range(256):
  out = int(pow( float(number)/255.0, gamma ) * 255.0)
  print(str(out), end='')
  if number != 255:
    print(",", end='')
print("];")


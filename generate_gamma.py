import argparse

parser = argparse.ArgumentParser(description="Generate an 8 bit gamma remap table for rust code")
parser.add_argument('--gamma', dest='gamma', action='store', required=True );

args = parser.parse_args()
gamma = float(args.gamma);

print("const gamma : [u16; 257 ] = [", end='')

for number in range(257):
  out = int(pow( float(number)/256.0, gamma ) * 256.0)
  print(str(out), end='')
  if number != 256:
    print(",", end='')
print("];")


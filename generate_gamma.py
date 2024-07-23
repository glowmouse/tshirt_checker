import argparse

parser = argparse.ArgumentParser(description="Generate an 8 bit gamma remap table for rust code")
parser.add_argument('--gamma', dest='gamma', action='store', required=True );

args = parser.parse_args()
gamma = float(args.gamma);

print("const gamma : [u16; 1025] = [", end='')

for number in range(1025):
  out = int(pow( float(number)/1024.0, gamma ) * 1024.0)
  print(str(out), end='')
  if number != 1024:
    print(",", end='')
print("];")


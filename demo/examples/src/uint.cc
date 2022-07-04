#include <cassert>
#include <defi-wallet-core-cpp/src/uint.rs.h>
#include <rust/cxx.h>
using namespace org::defi_wallet_core;

int main(int argc, char *argv[]) {
  assert(u256("15") == u256("15", 10));
  assert(u256("15") == u256("0xf", 16));
  assert(u256("1000") == u256("100").add(u256("900")));
  assert(u256("999999999999999999999999300") ==
         u256("1000000000000000000000000000").sub(u256("700")));
  assert(u256("199999999999999999980000200") ==
         u256("99999999999999999990000100").mul(u256("2")));
  assert(u256("1999999999999999999800002") ==
         u256("199999999999999999980000200").div(u256("100")));
  assert(u256("800002") ==
         u256("1999999999999999999800002").rem(u256("1000000")));
  assert(u256("512003840009600008") == u256("800002").pow(u256("3")));
  assert(u256("512003840009600008").neg() ==
         u256_max_value().sub(u256("512003840009600007")));
  return 0;
}

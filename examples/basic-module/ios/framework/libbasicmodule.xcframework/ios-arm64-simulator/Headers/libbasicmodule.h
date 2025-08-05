#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct String;

extern "C" {

double numericMethod(double arg);

String stringMethod(String arg);

bool booleanMethod(bool arg);

}  // extern "C"

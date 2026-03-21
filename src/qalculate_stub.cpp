#include <cstdlib>
#include <cstring>
#include <libqalculate/Calculator.h>
#include <string>

extern "C" char *qalculate_stub_calculate(const char *expression) {
  Calculator *calculator = new Calculator();
  delete calculator;

  std::string result = "qalculate stub: ";
  if (expression != nullptr) {
    result += expression;
  }

  char *output = static_cast<char *>(std::malloc(result.size() + 1));
  if (output == nullptr) {
    return nullptr;
  }

  std::memcpy(output, result.c_str(), result.size() + 1);
  return output;
}

extern "C" void qalculate_stub_free_string(char *value) { std::free(value); }

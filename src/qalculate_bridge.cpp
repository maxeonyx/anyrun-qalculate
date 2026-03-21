#include <cstdlib>
#include <cstring>
#include <libqalculate/Calculator.h>
#include <string>

namespace {

struct QalculateHandle {
  Calculator calculator;
  EvaluationOptions evaluation_options;
  PrintOptions print_options;

  QalculateHandle() {
    calculator.loadExchangeRates();
    calculator.loadGlobalDefinitions();
    print_options.number_fraction_format = FRACTION_DECIMAL;
  }
};

char *copy_string(const std::string &value) {
  char *output = static_cast<char *>(std::malloc(value.size() + 1));
  if (output == nullptr) {
    return nullptr;
  }

  std::memcpy(output, value.c_str(), value.size() + 1);
  return output;
}

} // namespace

extern "C" void *qalculate_new() {
  return new QalculateHandle();
}

extern "C" void qalculate_free(void *handle) {
  delete static_cast<QalculateHandle *>(handle);
}

extern "C" char *qalculate_calculate(void *handle, const char *expression) {
  if (handle == nullptr || expression == nullptr) {
    return nullptr;
  }

  auto *qalculate = static_cast<QalculateHandle *>(handle);
  std::string result = qalculate->calculator.calculateAndPrint(
      expression, 10000, qalculate->evaluation_options, qalculate->print_options);

  if (result.empty()) {
    return nullptr;
  }

  return copy_string(result);
}

extern "C" void qalculate_free_string(char *value) { std::free(value); }

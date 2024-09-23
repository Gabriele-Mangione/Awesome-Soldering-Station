#pragma once
#include <algorithm>
#include <cmath>

#define ABS(a) a < 0 ? -a : a

template <typename T, size_t S> class RingedBuffer {
private:
  size_t index;
  T buffer[S] = {0};
  bool isEmpty = true;

public:
  RingedBuffer() {}
  T avg() {
    T res = 0;
    for (size_t i = 0; i < S; i++) {
      res += buffer[i] / (float)S;
    }
    return res;
  }
  void push(T value) {
    if (isEmpty) {
      fill(value);
      isEmpty = false;
    }
    buffer[index] = value;
    index = (index + 1) % 50;
  }
  void fill(T value) {
    for (size_t i = 0; i < S; i++) {
      buffer[i] = value;
    }
  }

  T gaussAvg() {
    T cBuffer[S];
    memcpy(cBuffer, buffer, S * sizeof(T));
    std::sort(cBuffer, buffer + S * sizeof(T));
    T res = 0;

    size_t center = S / 2;

    T coeff = 0;
    for (size_t i = 0; i < S; i++) {
      T weight = sqrt(ABS(i - center)) + 1;
      res += cBuffer[i] / weight;
    }
    return res / coeff;
  }

  T getCumulDelta() {
    T delta = 0;
    for (size_t i = 1; i < S; i++) {
        delta += abs(buffer[i] - buffer[i-1]);
    }
    return delta;
  }
};

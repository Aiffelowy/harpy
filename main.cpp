#include <iostream>

int main() {
  int a = 1;
  int& sus = a;

  {
    int b = 2;
    sus = b;
  }

  std::cout << sus;
}

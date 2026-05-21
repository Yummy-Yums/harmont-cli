cmake_minimum_required(VERSION 3.20)
project(harmont_example_c C)
set(CMAKE_C_STANDARD 11)

add_library(addlib src/main.c)

enable_testing()
add_executable(test_main tests/test_main.c)
target_link_libraries(test_main PRIVATE addlib)
add_test(NAME test_main COMMAND test_main)

cmake_minimum_required(VERSION 3.20)
project(harmont_example_cpp CXX)
set(CMAKE_CXX_STANDARD 20)

add_library(addlib src/main.cpp)

enable_testing()
add_executable(test_main tests/test_main.cpp)
target_link_libraries(test_main PRIVATE addlib)
add_test(NAME test_main COMMAND test_main)

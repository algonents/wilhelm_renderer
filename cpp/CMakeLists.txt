cmake_minimum_required(VERSION 3.27)
project(glrenderer)

set(CMAKE_CXX_STANDARD 17)

find_package(OpenGL REQUIRED)

# === GLFW Setup ===
set(GLFW_BUILD_DOCS OFF CACHE BOOL "" FORCE)
set(GLFW_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(GLFW_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)

# Enable both Wayland and X11 support (GLFW selects backend at runtime)
set(GLFW_BUILD_WAYLAND ON CACHE BOOL "" FORCE)
set(GLFW_BUILD_X11 ON CACHE BOOL "" FORCE)

add_subdirectory(glfw-3.4)

# === FreeType Setup (bundled) ===
set(FT_DISABLE_ZLIB ON CACHE BOOL "" FORCE)
set(FT_DISABLE_BZIP2 ON CACHE BOOL "" FORCE)
set(FT_DISABLE_PNG ON CACHE BOOL "" FORCE)
set(FT_DISABLE_HARFBUZZ ON CACHE BOOL "" FORCE)
set(FT_DISABLE_BROTLI ON CACHE BOOL "" FORCE)

add_subdirectory(freetype-2.13.2)

include_directories(glad/include)
include_directories(glfw-3.4/include)
include_directories(freetype-2.13.2/include)

add_library(glrenderer STATIC glrenderer.cpp glad/src/glad.c)

target_link_libraries(glrenderer glfw)
target_link_libraries(glrenderer freetype)
target_link_libraries(glrenderer OpenGL::GL)

# Export static libraries
set_target_properties(glfw PROPERTIES ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}")
set_target_properties(freetype PROPERTIES ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}")

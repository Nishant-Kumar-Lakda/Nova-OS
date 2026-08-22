cmake_minimum_required(VERSION 3.22.1)
project(nova_android_llama LANGUAGES C CXX)

set(LLAMA_CPP_DIR "${CMAKE_CURRENT_LIST_DIR}/llama.cpp" CACHE PATH "Pinned llama.cpp source tree")

if(NOT EXISTS "${LLAMA_CPP_DIR}/CMakeLists.txt")
    message(FATAL_ERROR "llama.cpp source tree is missing. Run scripts/prepare_android_model.ps1 on Windows or use the NOVA Android Native GitHub workflow.")
endif()

set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(LLAMA_BUILD_COMMON OFF CACHE BOOL "" FORCE)
set(LLAMA_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(LLAMA_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(LLAMA_BUILD_SERVER OFF CACHE BOOL "" FORCE)
set(LLAMA_OPENSSL OFF CACHE BOOL "" FORCE)
set(GGML_OPENMP OFF CACHE BOOL "" FORCE)
set(GGML_NATIVE OFF CACHE BOOL "" FORCE)
set(GGML_LLAMAFILE OFF CACHE BOOL "" FORCE)
set(GGML_BACKEND_DL OFF CACHE BOOL "" FORCE)

add_subdirectory("${LLAMA_CPP_DIR}" "${CMAKE_CURRENT_BINARY_DIR}/llama.cpp" EXCLUDE_FROM_ALL)

add_library(nova_llama SHARED nova_llama.cpp)

target_compile_features(nova_llama PRIVATE cxx_std_17)
target_include_directories(nova_llama PRIVATE
    "${LLAMA_CPP_DIR}/include"
    "${LLAMA_CPP_DIR}/ggml/include"
)

target_link_libraries(nova_llama PRIVATE llama log android)

target_compile_definitions(nova_llama PRIVATE NOVA_LLAMA_MODEL_NAME="SmolLM2-135M-Instruct-Q2_K.gguf")

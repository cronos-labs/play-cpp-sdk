cmake_minimum_required(VERSION 3.10)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQURIED ON)
set(CMAKE_OSX_ARCHITECTURES "x86_64" CACHE INTERNAL "" FORCE)
if (APPLE)
  set(CMAKE_CXX_FLAGS "-framework Security -framework CoreFoundation")
endif()

project(demo VERSION 1.0)

# Run pre_build.bat in configuration step
if (WIN32)
  execute_process(
    COMMAND cmd /c ${CMAKE_CURRENT_SOURCE_DIR}/pre_build.bat
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  )
endif()

# Add subdirectory sdk in configuration step
add_subdirectory(sdk)

# add static demo
add_executable(demostatic main.cc chainmain.cc cronos.cc extra.cc third_party/easywsclient/easywsclient.cpp)
if (WIN32)
  target_link_libraries(demostatic PUBLIC ${PLAY_CPP_SDK_LIB} Ncrypt userenv ntdll Secur32 crypt32)
endif()
if (APPLE)
  target_link_libraries(demostatic PUBLIC ${PLAY_CPP_SDK_LIB})
endif()
if (UNIX AND NOT APPLE)
  target_link_libraries(demostatic PUBLIC ${PLAY_CPP_SDK_LIB} pthread dl)
endif()

# add dynamic demo
add_executable(demo main.cc chainmain.cc cronos.cc extra.cc third_party/easywsclient/easywsclient.cpp)
if (WIN32)
  target_link_libraries(demo PUBLIC play_cpp_sdk)
  # Copy dll
  add_custom_command(TARGET demo
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy ${PLAY_CPP_SDK_DLL} ${demo_BINARY_DIR}
  )
endif()
if (APPLE)
  # link library play_cpp_sdk built from subdirectory
  target_link_libraries(demo PUBLIC play_cpp_sdk)
endif()
if (UNIX AND NOT APPLE)
  # link library play_cpp_sdk built from subdirectory
  target_link_libraries(demo PUBLIC play_cpp_sdk)
endif()

add_subdirectory(examples)

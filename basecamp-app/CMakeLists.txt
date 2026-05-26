cmake_minimum_required(VERSION 3.16)
project(Lp0002PrivateMultisigBasecamp VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Widgets Quick Qml QuickWidgets)

add_library(lp0002_private_multisig SHARED
    src/lp0002_plugin.cpp
    src/lp0002_plugin.h
    src/lp0002_widget.cpp
    src/lp0002_widget.h
    resources.qrc
)

target_include_directories(lp0002_private_multisig PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

target_link_libraries(lp0002_private_multisig PRIVATE
    Qt6::Core
    Qt6::Widgets
    Qt6::Quick
    Qt6::Qml
    Qt6::QuickWidgets
)

set_target_properties(lp0002_private_multisig PROPERTIES
    OUTPUT_NAME "lp0002_private_multisig"
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/modules
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/modules
)

add_custom_command(TARGET lp0002_private_multisig POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
            ${CMAKE_CURRENT_SOURCE_DIR}/metadata.json
            $<TARGET_FILE_DIR:lp0002_private_multisig>/metadata.json
    COMMENT "Copying Basecamp module metadata"
)

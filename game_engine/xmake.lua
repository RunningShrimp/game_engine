-- xmake.lua
-- Game Engine - Cross-platform Build Configuration
--
-- This is the main xmake configuration file for the game engine project.
-- It provides cross-platform build support for Windows, Linux, macOS, Android, and WebAssembly.
--
-- Quick Start:
--   xmake                    # Build the project
--   xmake run                # Run the game
--   xmake f -p android       # Configure for Android
--   xmake f -p wasm          # Configure for WebAssembly
--   xmake -vD                # Build with debug info
--   xmake clean              # Clean build artifacts
--
-- For more information, see docs/xmake_build_guide.md

set_project("game-engine")
set_version("0.1.0")

-- ============================================================================
-- Configuration Options
-- ============================================================================

-- Enable Rust support
set_languages("c++20", "rust")

-- Add configuration modes
add_rules("mode.debug", "mode.release")
add_rules("mode.asan", "mode.tsan", "mode.lsan", "mode.ubsan")

-- ============================================================================
-- Common Settings
-- ============================================================================

-- Set default optimization flags
if is_mode("release") then
    set_optimize("fastest")
    set_symbols("hidden")
    set_strip("all")
elseif is_mode("debug") then
    set_symbols("debug")
    set_optimize("none")
    add_defines("DEBUG", "_DEBUG")
end

-- ============================================================================
-- Platform Detection
-- ============================================================================

local platform_vars = {}
if is_plat("windows") then
    platform_vars = {
        defines = "WINDOWS",
        ldflags = "/SUBSYSTEM:WINDOWS"
    }
elseif is_plat("linux") then
    platform_vars = {
        defines = "LINUX",
        ldflags = "-pthread"
    }
elseif is_plat("macosx") then
    platform_vars = {
        defines = "MACOS",
        ldflags = "-framework Cocoa -framework Metal"
    }
elseif is_plat("android") then
    platform_vars = {
        defines = "ANDROID",
        ldflags = "-landroid -llog"
    }
elseif is_plat("wasm") then
    platform_vars = {
        defines = "WASM",
        ldflags = "-s USE_SDL=2 -s WASM=1"
    }
end

-- ============================================================================
-- Game Engine Library Target
-- ============================================================================

target("game-engine-core")
    -- Static library for core engine
    set_kind("static")
    add_files("src/lib.rs", {rootdir = "game_engine"})

    -- Add Rust source files
    add_files("src/**/*.rs", {rootdir = "game_engine"})

    -- Platform-specific defines
    add_defines(platform_vars.defines)

    -- Rust features
    add_defines("RUST_PREFIX=\"game_engine\"")

    -- Include directories
    add_includedirs("include", {public = true})

    -- Dependencies (will be linked via Cargo.toml)
    -- Note: xmake delegates Rust dependencies to Cargo

    -- Link syslibraries
    if is_plat("linux") then
        add_syslinks("pthread", "dl", "m")
    elseif is_plat("windows") then
        add_syslinks("ws2_32", "userenv", "msvcrt")
    elseif is_plat("macosx") then
        add_frameworks("Cocoa", "Metal", "CoreVideo")
    end

    -- Installation
    on_install(function (target)
        os.cp("$(targetdir)/$(filename).a", "$(installir)/lib/")
    end)

target_end()

-- ============================================================================
-- Game Executable Target
-- ============================================================================

target("game")
    -- Binary executable
    set_kind("binary")
    add_files("src/main.rs", {rootdir = "game_engine"})

    -- Link against engine core
    add_deps("game-engine-core")

    -- Platform-specific configuration
    if is_plat("windows") then
        add_ldflags("/SUBSYSTEM:CONSOLE", {force = true})
    elseif is_plat("macosx") then
        add_ldflags("-framework Cocoa -framework Metal")
    elseif is_plat("linux") then
        add_ldflags("-pthread -ldl -lm")
    elseif is_plat("android") then
        add_ldflags("-landroid -llog")
    end

    -- Post-build: Copy assets
    after_build(function (target)
        local assets_dir = path.absolute("assets")
        local target_dir = path.absolute(target:targetdir())

        -- Check if assets directory exists
        if os.isdir(assets_dir) then
            local target_assets = path.join(target_dir, "assets")
            os.cp(assets_dir, target_assets)

            -- Verbose output
            if is_mode("debug") then
                print("Assets copied to: " .. target_assets)
            end
        end
    end)

    -- Installation
    on_install(function (target)
        -- Install binary
        os.cp("$(targetdir)/game", "$(installir)/bin/")

        -- Install assets if they exist
        if os.isdir("$(targetdir)/assets") then
            os.cp("$(targetdir)/assets", "$(installir)/share/")
        end
    end)

target_end()

-- ============================================================================
-- Asset Processing Target
-- ============================================================================

target("game-resources")
    -- Phony target for resource processing
    set_kind("phony")

    -- Resource processing script
    on_build(function (target)
        local assets_dir = "assets"
        local build_dir = "$(buildir)/assets"

        -- Create build directory
        os.mkdir(build_dir)

        -- Copy assets if directory exists
        if os.isdir(assets_dir) then
            print("Processing assets...")

            -- Copy all assets
            os.cp(assets_dir .. "/**", build_dir)

            -- Optionally compress assets
            if is_mode("release") then
                print("Compressing assets...")
                local asset_zip = "$(buildir)/assets.zip"
                os.exec("zip -r %s %s", asset_zip, build_dir)
            end

            print("Assets processed successfully!")
        else
            print("Warning: assets/ directory not found, skipping resource processing")
        end
    end)

target_end()

-- ============================================================================
-- Toolchains for Cross-Compilation
-- ============================================================================

-- Android ARM64 toolchain
toolchain("android-arm64")
    set_kind("standalone")
    set_sdkdir(os.getenv("ANDROID_NDK_HOME") or "~/Android/Sdk/ndk")
    set_arch("arm64-v8a")

    -- Set cross compiler
    set_toolset("cc", "aarch64-linux-android-clang")
    set_toolset("cxx", "aarch64-linux-android-clang++")
    set_toolset("ld", "aarch64-linux-android-ld")
    set_toolset("ar", "aarch64-linux-android-ar")

    -- Add flags
    add_cxxflags("-fPIC", "-fdata-sections", "-ffunction-sections")
    add_ldflags("-fPIC", "-Wl,--gc-sections")

toolchain_end()

-- Android ARMv7 toolchain
toolchain("android-armv7-a")
    set_kind("standalone")
    set_sdkdir(os.getenv("ANDROID_NDK_HOME") or "~/Android/Sdk/ndk")
    set_arch("armeabi-v7a")

    -- Set cross compiler
    set_toolset("cc", "arm-linux-androideabi-clang")
    set_toolset("cxx", "arm-linux-androideabi-clang++")
    set_toolset("ld", "arm-linux-androideabi-ld")
    set_toolset("ar", "arm-linux-androideabi-ar")

    -- Add flags
    add_cxxflags("-fPIC", "-march=armv7-a", "-mfpu=neon")
    add_ldflags("-fPIC", "-Wl,--fix-cortex-a8")

toolchain_end()

-- WebAssembly toolchain
toolchain("wasm")
    set_kind("standalone")
    set_sdkdir(os.getenv("EMSCRIPTEN_ROOT") or os.getenv("EMSDK"))

    -- Emscripten toolset
    set_toolset("cc", "emcc")
    set_toolset("cxx", "em++")
    set_toolset("ld", "emcc")
    set_toolset("ar", "emar")

    -- WASM flags
    add_cxxflags("-fPIC")
    add_ldflags("-s WASM=1", "-s USE_SDL=2", "-s ALLOW_MEMORY_GROWTH=1")

toolchain_end()

-- ============================================================================
-- Custom Tasks
-- ============================================================================

-- Task: Clean everything
task("clean-all")
    on_run(function ()
        -- Clean build artifacts
        os.exec("xmake clean")

        -- Clean profiling data
        if os.isdir("profiling_data") then
            os.rm("profiling_data/*.dat.gz")
        end

        -- Clean build directory
        if os.isdir("build") then
            os.rm("build/**")
        end

        print("Clean completed!")
    end)
task_end()

-- Task: Format code
task("format")
    on_run(function ()
        print("Formatting Rust code...")
        os.exec("cargo fmt")

        print("Formatting completed!")
    end)
task_end()

-- Task: Run linter
task("lint")
    on_run(function ()
        print("Running Rust linter...")
        os.exec("cargo clippy -- -D warnings")

        print("Linting completed!")
    end)
task_end()

-- Task: Run tests
task("test")
    on_run(function ()
        print("Running tests...")
        os.exec("cargo test --all")

        print("Tests completed!")
    end)
task_end()

-- Task: Generate documentation
task("docs")
    on_run(function ()
        print("Generating documentation...")
        os.exec("cargo doc --no-deps --open")

        print("Documentation generated!")
    end)
task_end()

-- ============================================================================
-- Build Configurations
-- ============================================================================

-- Debug configuration
config("debug")
    set_mode("debug")
    set_symbols("debug")
    set_warnings("all")
    add_defines("DEBUG", "_DEBUG")
config_end()

-- Release configuration
config("release")
    set_mode("release")
    set_optimize("fastest")
    set_symbols("hidden")
    set_strip("all")
    add_defines("NDEBUG", "_NDEBUG")
config_end()

-- ============================================================================
-- Package Configuration
-- ============================================================================

-- Task: Create distribution package
task("package")
    on_run(function ()
        local version = "v0.1.0"
        local dist_dir = "dist"

        -- Create distribution directory
        os.mkdir(dist_dir)

        -- Build release version
        os.exec("xmake config -m release")
        os.exec("xmake")

        -- Copy binaries
        os.mkdir(path.join(dist_dir, "bin"))
        os.cp("build/ release/game", path.join(dist_dir, "bin"))

        -- Copy assets
        if os.isdir("assets") then
            os.cp("assets", path.join(dist_dir, "share"))
        end

        -- Copy documentation
        if os.isdir("docs") then
            os.cp("docs", path.join(dist_dir, "doc"))
        end

        -- Create archive
        local archive_name = "game-engine-" .. version .. "-" .. os.host() .. "-" .. os.arch()
        if is_plat("windows") then
            os.exec("zip -r " .. archive_name .. ".zip " .. dist_dir)
        else
            os.exec("tar -czf " .. archive_name .. ".tar.gz " .. dist_dir)
        end

        print("Package created: " .. archive_name)
    end)
task_end()

-- ============================================================================
-- Installation Rules
-- ============================================================================

-- Task: Install
task("install")
    on_run(function ()
        local install_prefix = os.getenv("INSTALL_PREFIX") or "/usr/local"

        print("Installing to: " .. install_prefix)

        -- Create directories
        os.mkdir(path.join(install_prefix, "bin"))
        os.mkdir(path.join(install_prefix, "lib"))
        os.mkdir(path.join(install_prefix, "share"))

        -- Copy files
        if os.isdir("build") then
            os.cp("build/*/game", path.join(install_prefix, "bin"))
        end

        if os.isdir("assets") then
            os.cp("assets", path.join(install_prefix, "share"))
        end

        print("Installation completed!")
    end)
task_end()

-- ============================================================================
-- Uninstallation Rules
-- ============================================================================

-- Task: Uninstall
task("uninstall")
    on_run(function ()
        local install_prefix = os.getenv("INSTALL_PREFIX") or "/usr/local"

        print("Uninstalling from: " .. install_prefix)

        -- Remove files
        os.rm(path.join(install_prefix, "bin/game"))
        if os.isdir(path.join(install_prefix, "share/assets")) then
            os.rm(path.join(install_prefix, "share/assets"))
        end

        print("Uninstallation completed!")
    end)
task_end()

-- ============================================================================
-- CI/CD Support
-- ============================================================================

-- Task: CI Build
task("ci-build")
    on_run(function ()
        -- Set strict mode for CI
        os.setenv("RUST_BACKTRACE", "1")

        -- Build in release mode
        os.exec("xmake config -m release")
        os.exec("xmake -vD")

        -- Run tests
        os.exec("xmake test")

        -- Run linter
        os.exec("xmake lint")

        print("CI build completed successfully!")
    end)
task_end()

-- ============================================================================
-- Default Target
-- ============================================================================

-- Set default target to build
set_default("game")

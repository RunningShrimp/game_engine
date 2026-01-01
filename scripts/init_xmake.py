#!/usr/bin/env python3
"""
xmake交互式初始化工具

自动检测平台、编译器和依赖，生成最优配置。
"""

import os
import sys
import platform
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class Colors:
    """终端颜色"""
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    END = '\033[0m'
    BOLD = '\033[1m'


def print_header(text: str):
    """打印标题"""
    print(f"\n{Colors.HEADER}{Colors.BOLD}{'='*60}{Colors.END}")
    print(f"{Colors.HEADER}{Colors.BOLD}{text:^60}{Colors.END}")
    print(f"{Colors.HEADER}{Colors.BOLD}{'='*60}{Colors.END}\n")


def print_info(text: str):
    """打印信息"""
    print(f"{Colors.BLUE}ℹ{Colors.END} {text}")


def print_success(text: str):
    """打印成功信息"""
    print(f"{Colors.GREEN}✓{Colors.END} {text}")


def print_warning(text: str):
    """打印警告"""
    print(f"{Colors.YELLOW}⚠{Colors.END} {text}")


def print_error(text: str):
    """打印错误"""
    print(f"{Colors.RED}✗{Colors.END} {text}")


def detect_platform() -> Tuple[str, str]:
    """检测操作系统和架构"""
    system = platform.system().lower()
    machine = platform.machine().lower()

    # 映射到xmake平台名
    platform_map = {
        'darwin': 'macosx',
        'linux': 'linux',
        'windows': 'windows',
    }

    # 映射到xmake架构名
    arch_map = {
        'x86_64': 'x86_64',
        'amd64': 'x86_64',
        'arm64': 'arm64',
        'aarch64': 'arm64',
        'armv7': 'armv7',
        'i386': 'i386',
        'i686': 'i386',
    }

    plat = platform_map.get(system, system)
    arch = arch_map.get(machine, machine)

    return plat, arch


def detect_compiler(plat: str) -> Optional[str]:
    """检测编译器"""
    compilers = []

    if plat == 'windows':
        # Windows: 检测MSVC和MinGW
        try:
            result = subprocess.run(['cl'], capture_output=True, shell=True)
            if 'Microsoft' in result.stderr.decode():
                compilers.append(('msvc', 'MSVC (Visual Studio)'))
        except FileNotFoundError:
            pass

        try:
            result = subprocess.run(['gcc', '--version'], capture_output=True)
            if result.returncode == 0:
                compilers.append(('gcc', 'MinGW GCC'))
        except FileNotFoundError:
            pass

    elif plat in ['linux', 'macosx']:
        # Linux/macOS: 检测GCC和Clang
        try:
            result = subprocess.run(['clang', '--version'], capture_output=True)
            if result.returncode == 0:
                version = result.stdout.decode().split('\n')[0]
                compilers.append(('clang', f'Clang - {version}'))
        except FileNotFoundError:
            pass

        try:
            result = subprocess.run(['gcc', '--version'], capture_output=True)
            if result.returncode == 0:
                version = result.stdout.decode().split('\n')[0]
                compilers.append(('gcc', f'GCC - {version}'))
        except FileNotFoundError:
            pass

    return compilers[0] if compilers else None


def detect_xmake() -> bool:
    """检测xmake是否安装"""
    try:
        result = subprocess.run(['xmake', '--version'], capture_output=True)
        if result.returncode == 0:
            version = result.stdout.decode().strip()
            print_success(f"检测到xmake: {version}")
            return True
    except FileNotFoundError:
        print_error("未找到xmake")
        print_info("请访问 https://xmake.io/#/getting_started 安装xmake")
        return False


def check_dependencies(plat: str) -> Dict[str, bool]:
    """检查依赖库"""
    deps = {}

    if plat == 'linux':
        # Linux依赖
        required_packages = {
            'libx11-dev': 'X11开发库',
            'libgl1-mesa-dev': 'OpenGL开发库',
            'libxcursor-dev': '光标支持',
            'libxrandr-dev': 'RANDR扩展',
            'libxinerama-dev': 'Xinerama扩展',
            'libxi-dev': 'XInput扩展',
        }

        for package, description in required_packages.items():
            try:
                result = subprocess.run(['dpkg', '-l', package], capture_output=True)
                deps[description] = result.returncode == 0
            except FileNotFoundError:
                deps[description] = False

    elif plat == 'macosx':
        # macOS依赖（通常已安装）
        deps['Xcode'] = True
        try:
            result = subprocess.run(['xcode-select', '-p'], capture_output=True)
            if result.returncode != 0:
                deps['Xcode'] = False
        except FileNotFoundError:
            deps['Xcode'] = False

    elif plat == 'windows':
        # Windows依赖
        deps['Windows SDK'] = True  # 假设已安装

    return deps


def generate_config(plat: str, arch: str, mode: str = 'release') -> List[str]:
    """生成xmake配置命令"""
    config_cmds = [
        f'xmake f -p {plat} -a {arch} -m {mode}'
    ]

    # 平台特定配置
    if plat == 'macosx' and arch == 'arm64':
        config_cmds.append('--appledev=entitlements')

    elif plat == 'android':
        # Android额外配置
        ndk_path = os.environ.get('ANDROID_NDK_HOME')
        if ndk_path:
            config_cmds.append(f'--ndk={ndk_path}')

    return config_cmds


def suggest_config(plat: str, arch: str, compiler: Optional[str]) -> Dict:
    """建议配置"""
    config = {
        'platform': plat,
        'architecture': arch,
        'mode': 'release',
        'compiler': compiler,
        'optimization': 'fastest' if plat != 'wasm' else 'none',
    }

    # 根据平台调整建议
    if plat == 'wasm':
        config['optimization'] = 'none'
        config['mode'] = 'debug'

    elif plat in ['linux', 'macosx']:
        config['ccache'] = 'y'

    return config


def interactive_config() -> Dict:
    """交互式配置"""
    print_header("xmake配置向导")

    # 检测平台
    plat, arch = detect_platform()
    print_info(f"检测到平台: {plat} ({arch})")

    # 检测编译器
    compiler = detect_compiler(plat)
    if compiler:
        print_success(f"检测到编译器: {compiler[1]}")

    # 检查依赖
    print_info("检查依赖库...")
    deps = check_dependencies(plat)

    missing_deps = [name for name, installed in deps.items() if not installed]
    if missing_deps:
        print_warning("缺少以下依赖:")
        for dep in missing_deps:
            print(f"  - {dep}")

        if plat == 'linux':
            print_info(f"\n安装命令: sudo apt install {' '.join(missing_deps)}")

    # 生成建议配置
    config = suggest_config(plat, arch, compiler[0] if compiler else None)

    print_header("建议配置")
    print(f"  平台:      {config['platform']}")
    print(f"  架构:      {config['architecture']}")
    print(f"  模式:      {config['mode']}")
    print(f"  优化:      {config['optimization']}")
    if 'ccache' in config:
        print(f"  ccache:    {config['ccache']}")

    # 询问用户
    while True:
        response = input(f"\n是否接受此配置? [{Colors.GREEN}Y{Colors.END}/n]: ").strip().lower()
        if not response or response == 'y' or response == 'yes':
            return config
        elif response == 'n' or response == 'no':
            return custom_config(plat, arch)
        else:
            print_error("无效输入，请输入 Y 或 n")


def custom_config(plat: str, arch: str) -> Dict:
    """自定义配置"""
    print_header("自定义配置")

    config = {'platform': plat, 'architecture': arch}

    # 选择模式
    print("\n构建模式:")
    print("  1. release (生产版本)")
    print("  2. debug   (调试版本)")
    print("  3. asan    (Address Sanitizer)")
    print("  4. tsan    (Thread Sanitizer)")

    mode_choice = input("选择模式 [1-4]: ").strip()
    mode_map = {'1': 'release', '2': 'debug', '3': 'asan', '4': 'tsan'}
    config['mode'] = mode_map.get(mode_choice, 'release')

    # 选择优化级别
    if config['mode'] == 'release':
        print("\n优化级别:")
        print("  1. fastest (最快)")
        print("  2. faster  (较快)")
        print("  3. fast    (一般)")
        print("  4. none    (无优化)")

        opt_choice = input("选择优化级别 [1-4]: ").strip()
        opt_map = {'1': 'fastest', '2': 'faster', '3': 'fast', '4': 'none'}
        config['optimization'] = opt_map.get(opt_choice, 'fastest')
    else:
        config['optimization'] = 'none'

    # 启用ccache
    ccache = input("\n启用ccache? [Y/n]: ").strip().lower()
    config['ccache'] = 'y' if (not ccache or ccache == 'y' or ccache == 'yes') else 'n'

    return config


def apply_config(config: Dict):
    """应用配置"""
    print_header("应用配置")

    config_dir = Path('.xmake')
    config_dir.mkdir(exist_ok=True)

    # 生成配置命令
    config_cmds = generate_config(config['platform'], config['architecture'], config['mode'])

    # 添加额外选项
    if 'ccache' in config and config['ccache'] == 'y':
        config_cmds[0] += ' --ccache=y'

    # 执行配置
    print_info("执行配置命令...")
    for cmd in config_cmds:
        print(f"  $ {cmd}")
        try:
            result = subprocess.run(cmd.split(), capture_output=True, text=True)
            if result.returncode == 0:
                print_success("配置成功!")
            else:
                print_error(f"配置失败: {result.stderr}")
                return False
        except Exception as e:
            print_error(f"执行错误: {e}")
            return False

    return True


def show_next_steps(config: Dict):
    """显示后续步骤"""
    print_header("后续步骤")

    print(f"1. 构建项目:")
    print(f"   {Colors.CYAN}xmake{Colors.END}")
    print()

    print(f"2. 运行项目:")
    print(f"   {Colors.CYAN}xmake run{Colors.END}")
    print()

    print(f"3. 清理构建:")
    print(f"   {Colors.CYAN}xmake clean{Colors.END}")
    print()

    if config['platform'] == 'android':
        print(f"4. 生成APK:")
        print(f"   {Colors.CYAN}xmake package{Colors.END}")
        print()

    print(f"更多命令: {Colors.CYAN}xmake help{Colors.END}")
    print(f"配置菜单: {Colors.CYAN}xmake f --menu{Colors.END}")


def main():
    """主函数"""
    print_header("Game Engine xmake初始化向导")

    # 检测xmake
    if not detect_xmake():
        sys.exit(1)

    # 交互式配置
    config = interactive_config()

    # 应用配置
    if apply_config(config):
        show_next_steps(config)
        print_success("\n初始化完成!")
    else:
        print_error("\n初始化失败")
        sys.exit(1)


if __name__ == '__main__':
    main()

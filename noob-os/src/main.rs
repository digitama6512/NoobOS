// 不使用标准库
#![no_std]
// 不使用标准的主函数入口
#![no_main]

use core::arch::asm;

use limine::BaseRevision;
use limine::request::{
    FramebufferRequest, RequestsEndMarker, RequestsStartMarker,
};

// 强制编译器保留此变量（即使未被显式使用）
#[used]
// 强制将此变量放在 ELF 文件的 .requests 段
#[unsafe(link_section = ".requests")]
// 声明内核支持的 Limine 引导协议版本
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
// 向引导程序请求图形帧缓冲区，引导程序在初始化阶段会填充这个请求的响应数据（如内存地址、分辨率、像素格式等）
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

// 边界标记
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

// 字体位图 8x8
static FONT_8X8: [[u8; 8]; 11] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // ' '
    [0x42, 0x42, 0x7E, 0x42, 0x42, 0x00, 0x00, 0x00], // 'H'
    [0x7E, 0x40, 0x7C, 0x40, 0x7E, 0x00, 0x00, 0x00], // 'E'
    [0x40, 0x40, 0x40, 0x40, 0x7E, 0x00, 0x00, 0x00], // 'L'
    [0x40, 0x40, 0x40, 0x40, 0x7E, 0x00, 0x00, 0x00], // 'L'
    [0x3C, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00], // 'O'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // ' '
    [0x42, 0x42, 0x5A, 0x66, 0x42, 0x00, 0x00, 0x00], // 'W'
    [0x3C, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00], // 'O'
    [0x7C, 0x42, 0x7C, 0x48, 0x44, 0x00, 0x00, 0x00], // 'R'
    [0x78, 0x44, 0x42, 0x42, 0x78, 0x00, 0x00, 0x00], // 'D'
];

// 使用 no_mangle 标记这个函数，来对它禁用名称重整
#[unsafe(no_mangle)]
/// `kmain` 程序的入口点，extern "C" 表示这个函数使用C语言的ABI，，使其可以被引导加载程序调用
/// 在屏幕上显示 hello world
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    // 获取帧缓冲区信息
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for char_index in 0..11 {
                // 选择一个字体
                let char_data = FONT_8X8[char_index];
                for row in 0..8 {
                    for col in 0..8 {
                        // 判断字体位图中第row行第col列是否为 #
                        if (char_data[row] >> (7 - col)) & 1 != 0 {
                            // row + 30: 在屏幕坐标 (30, 0) 开始画，该字体中 # 在屏幕上对应的行数
                            // framebuffer.pitch(): 屏幕一行的字节数
                            // char_index * 8 + col: 该字体中 # 在屏幕上对应的列数
                            // (char_index * 8 + col) * 4: 一个像素用32bit表示，ARGB
                            // pixel_offset 为 # 在framebuffer对应的地址偏移
                            let pixel_offset = (row + 30)
                                * framebuffer.pitch() as usize
                                + (char_index * 8 + col) * 4;
                            unsafe {
                                // 在该地址上显示青色
                                framebuffer
                                    .addr()
                                    .add(pixel_offset as usize)
                                    .cast::<u32>()
                                    // 青色(ARGB)
                                    .write(0xFF00FFFF)
                            };
                        }
                    }
                }
            }
        }
    }

    hcf();
}

#[panic_handler]
/// `rust_panic` 定义 panic 处理函数，当程序发生 panic 时进入停机状态
/// 在 no_std 环境中，我们需要定义一个自己的 panic 处理函数
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hcf();
}

/// `hcf` 停机函数
fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
        }
    }
}

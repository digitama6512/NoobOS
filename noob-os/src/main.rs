// 不使用标准库
#![no_std]
// 不使用标准的主函数入口
#![no_main]

use core::arch::asm;
use core::ffi::CStr;

use limine::BaseRevision;
use limine::framebuffer::Framebuffer;
use limine::request::{
    FramebufferRequest, ModuleRequest, RequestsEndMarker, RequestsStartMarker,
};

// 强制编译器保留此变量（即使未被显式使用）
#[used]
// 强制将此变量放在 ELF 文件的 .requests 段
#[unsafe(link_section = ".requests")]
// 声明内核支持的 Limine 引导协议版本
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static MOUDULE_REQUEST: ModuleRequest = ModuleRequest::new();

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

/// `draw_string` 使用limine帧缓冲打印字符串，横着打印
/// - psf1module: 内核镜像中的psf1文件
/// - framebuffer: limine帧缓冲
/// - x: 打印字符串的起始x坐标
/// - color: 打印字符串的颜色
unsafe fn draw_string(
    psf1module: &&limine::file::File,
    framebuffer: &Framebuffer<'_>,
    string: &CStr,
    x: usize,
    color: u32,
) {
    // 跳过psf1文件头部信息
    let glyphs_ptr = unsafe { psf1module.addr().add(4) };
    // 每个字形是8x16位图
    let row_size: usize = 16;
    let col_size: usize = 8;
    for (char_count, achar) in string.to_bytes().iter().enumerate() {
        // 当前字符在psf1文件中的字形
        let current_glyphs = unsafe { glyphs_ptr.add(16 * (*achar as usize)) };

        for row in 0..row_size {
            let row_of_glyphs = unsafe { *(current_glyphs.add(row)) };
            for col in 0..col_size {
                // 判断字形位图中第row行第col列是否为 #
                if (row_of_glyphs >> (7 - col)) & 1 != 0 {
                    let pixel_offset = (row + x) * framebuffer.pitch() as usize
                        + (char_count * 8 + col) * 4;
                    unsafe {
                        framebuffer
                            .addr()
                            .add(pixel_offset)
                            .cast::<u32>()
                            .write(color)
                    };
                }
            }
        }
    }
}

// 使用 no_mangle 标记这个函数，来对它禁用名称重整
#[unsafe(no_mangle)]
/// `kmain` 程序的入口点，extern "C" 表示这个函数使用C语言的ABI，，使其可以被引导加载程序调用
/// 在屏幕上显示 hello world
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    // 获取帧缓冲区信息
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            // 获取font
            if let Some(modules) = MOUDULE_REQUEST.get_response() {
                for amodule in modules.modules() {
                    if amodule.string() == c"zap-light16.psf" {
                        unsafe {
                            draw_string(
                                amodule,
                                &framebuffer,
                                c"Hello World",
                                0,
                                // 青色
                                0xFF00FFFF,
                            )
                        };
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

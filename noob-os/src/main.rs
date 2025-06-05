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

// 使用 no_mangle 标记这个函数，来对它禁用名称重整
#[unsafe(no_mangle)]
/// `kmain` 程序的入口点，extern "C" 表示这个函数使用C语言的ABI，，使其可以被引导加载程序调用
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    // 这段代码会在屏幕的 左上角 绘制一条 白色对角线（从 (0, 0) 到 (99, 99)）
    // 获取帧缓冲区信息
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        // 返回一个 迭代器，包含所有可用的帧缓冲区
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..100_u64 {
                // 选择择坐标 (i, i) 的像素
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // 写 0xFFFFFFFF(白色) 到坐标 (i, i) 的像素
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0xFFFFFFFF)
                };
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

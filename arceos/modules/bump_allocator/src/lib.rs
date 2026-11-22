#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    bytes_count: usize,
    used_pages: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            bytes_count: 0,
            used_pages: 0,
        }
    }
}

#[inline]
const fn align_up(x: usize, align: usize) -> usize {
    (x + align - 1) & !(align - 1)
}

#[inline]
const fn align_down(x: usize, align: usize) -> usize {
    x & !(align - 1)
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        // 起点向上、终点向下按页对齐，保证页区不越界
        let start = align_up(start, SIZE);
        let end = align_down(start + size, SIZE);
        self.start = start;
        self.end = end;
        self.b_pos = start;
        self.p_pos = end;
        self.bytes_count = 0;
        self.used_pages = 0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // bump 分配器只支持初始化时给定的单一区域，不支持后续追加
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> AllocResult<NonNull<u8>> {
        if layout.size() == 0 {
            return Ok(NonNull::dangling());
        }
        let align = layout.align();
        let addr = align_up(self.b_pos, align);
        let new_pos = addr + layout.size();
        // 与向下增长的页区相撞，说明空闲区不够
        if new_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        self.b_pos = new_pos;
        self.bytes_count += 1;
        Ok(NonNull::new(addr as *mut u8).unwrap())
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        // bump 不回收单个小块；计数减一，全部释放完才整体退回起点
        if self.bytes_count > 0 {
            self.bytes_count -= 1;
        }
        if self.bytes_count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> AllocResult<usize> {
        if num_pages == 0 {
            return Err(AllocError::InvalidParam);
        }
        if align_pow2 % SIZE != 0 || !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        // 从高地址端向下对齐，再向下切出 num_pages 页
        let addr = align_down(self.p_pos, align_pow2);
        let new_pos = addr - num_pages * SIZE;
        if new_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        self.p_pos = new_pos;
        self.used_pages += num_pages;
        Ok(addr)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // 页区永不释放，空实现
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages() - self.used_pages
    }
}
